#!/usr/bin/env python3
"""Mock Home Assistant Supervisor API used by the parity tests.

It records every request (method, path, query, selected headers, body) and
answers with a response *preset*.  It can be used in two ways:

* imported (``from mock_supervisor import MockSupervisor``) by ``parity.py``
* standalone, for manual poking:

      python3 tests/mock_supervisor.py --port 8123 --preset data
      SUPERVISOR_ENDPOINT=127.0.0.1:8123 target/debug/ha dns info

Preset syntax (``<kind>[:<argument>]``):

  ok                    200, {"result":"ok","data":{}}
  data                  200, {"result":"ok","data":<built-in sample data>}
  data:<json>           200, {"result":"ok","data":<json>}
  json:<json>           200, <json> as the complete body (application/json)
  file:<name>           200, body read from tests/parity/fixtures/<name>
                        (content type from extension: .json -> JSON, else text)
  error                 400, {"result":"error","message":"Something went wrong"}
  error:<status>        same JSON error body with another status code
  error:<status>:<msg>  JSON error with a custom message
  text:<status>         <status>, text/plain body "plain text body\n"
  text:<status>:<body>  <status>, text/plain with the given body (\n escapes ok)
  empty:<status>        <status>, no body and no Content-Type
  logs                  200, text/plain chunked multi-line log stream
  raw:<status>:<content-type>:<body>   fully custom
  hang[:<seconds>]      never answers (for manual timeout tests); the
                        connection is closed after <seconds> (default 3600)

Routes ("METHOD /path" -> preset) take precedence over the default preset; a
route value may hold several presets separated by " ;; " which are served in
order (the last one repeats).
"""

import argparse
import json
import os
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

FIXTURES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "parity", "fixtures")

# Headers that are part of the recorded request (lower-case).  Everything else
# (User-Agent, Host, Accept-Encoding, Connection, ...) is transport noise.
RECORDED_HEADERS = ("accept", "authorization", "content-type", "range", "content-length")

SAMPLE_DATA = {
    "version": "2025.11.1",
    "version_latest": "2025.11.2",
    "update_available": True,
    "host": "172.30.32.3",
    "servers": ["dns://8.8.8.8", "dns://1.1.1.1"],
    "locals": [],
    "empty_map": {},
    "fallback": False,
    "nothing": None,
    "port": 8123,
    "ratio": 0.25,
    "big": 12345678901234567890,
    "exp": 1e21,
    "negative": -3,
    "numeric_string": "123",
    "bool_string": "true",
    "null_string": "null",
    "empty_string": "",
    "tilde": "~",
    "date_string": "2025-11-01",
    "time_string": "12:30:45",
    "colon_string": "key: value",
    "hash_string": "a #comment",
    "leading_space": " leading",
    "trailing_space": "trailing ",
    "quote_string": "it's \"quoted\"",
    "dash_string": "- item",
    "star": "*",
    "at": "@home",
    "unicode": "Zürich ☃ 😀",
    "html": "<b>&</b>",
    "tab": "a\tb",
    "multiline": "line one\nline two\n",
    "multiline_noeol": "line one\n  indented\nline three",
    "long_text": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor "
    "incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud "
    "exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    "long_quoted": "true: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod "
    "tempor incididunt ut labore et dolore magna aliqua.",
    "nested": {
        "a10": 1,
        "a2": 2,
        "a1": 3,
        "B": "upper",
        "b": "lower",
        "_under": True,
        "1": "one",
        "list_of_maps": [
            {"name": "first", "slug": "core_ssh", "items": [1, 2, 3]},
            {"name": "second", "slug": "core_mosquitto", "items": []},
        ],
        "list_of_lists": [[1, 2], [], ["x"]],
    },
}

LOG_LINES = [
    "2025-11-11 10:00:00.000 INFO (MainThread) [supervisor.core] Starting\n",
    "2025-11-11 10:00:01.000 WARNING (MainThread) [supervisor.dns] Zürich ☃\n",
    "\x1b[32m2025-11-11 10:00:02.000 INFO coloured line\x1b[0m\n",
    "last line without newline",
]


def unescape(text):
    return text.replace("\\n", "\n").replace("\\t", "\t")


class Response:
    def __init__(self, status=200, content_type="application/json", body=b"", chunks=None, hang=0):
        self.status = status
        self.content_type = content_type
        self.body = body
        self.chunks = chunks
        self.hang = hang


def parse_preset(preset):
    """Turn a preset string into a Response."""
    preset = preset.strip()
    kind, _, arg = preset.partition(":")
    j = "application/json"
    if kind == "ok":
        return Response(200, j, b'{"result":"ok","data":{}}')
    if kind == "data":
        data = json.loads(arg) if arg else SAMPLE_DATA
        return Response(200, j, json.dumps({"result": "ok", "data": data}).encode())
    if kind == "json":
        return Response(200, j, arg.encode())
    if kind == "file":
        with open(os.path.join(FIXTURES, arg), "rb") as fh:
            body = fh.read()
        return Response(200, j if arg.endswith(".json") else "text/plain", body)
    if kind == "error":
        status, _, message = arg.partition(":")
        body = {"result": "error", "message": message or "Something went wrong"}
        return Response(int(status or 400), j, json.dumps(body).encode())
    if kind == "text":
        status, _, body = arg.partition(":")
        return Response(int(status or 200), "text/plain", unescape(body or "plain text body\\n").encode())
    if kind == "empty":
        return Response(int(arg or 200), None, b"")
    if kind == "logs":
        return Response(200, "text/plain", chunks=[line.encode() for line in LOG_LINES])
    if kind == "raw":
        status, content_type, body = arg.split(":", 2)
        return Response(int(status), content_type or None, unescape(body).encode())
    if kind == "hang":
        return Response(hang=float(arg or 3600))
    raise ValueError(f"unknown response preset: {preset!r}")


class MockSupervisor:
    """Threaded mock server.  ``requests`` holds the recorded requests."""

    def __init__(self, port=0, preset="ok", verbose=False):
        self.lock = threading.Lock()
        self.requests = []
        self.preset = preset
        self.routes = {}
        self.verbose = verbose
        mock = self

        class Handler(BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def log_message(self, *args):  # silence default logging
                pass

            def handle_any(self):
                if "chunked" in (self.headers.get("Transfer-Encoding") or "").lower():
                    # Go sends body-less POSTs as an empty chunked body.
                    body = b""
                    while True:
                        size = int(self.rfile.readline().split(b";")[0].strip() or b"0", 16)
                        if size == 0:
                            while self.rfile.readline() not in (b"\r\n", b"\n", b""):
                                pass
                            break
                        body += self.rfile.read(size)
                        self.rfile.readline()
                else:
                    length = int(self.headers.get("Content-Length") or 0)
                    body = self.rfile.read(length) if length else b""
                url = urlsplit(self.path)
                record = {
                    "method": self.command,
                    "path": url.path,
                    "query": url.query,
                    "headers": {
                        k.lower(): v for k, v in self.headers.items() if k.lower() in RECORDED_HEADERS
                    },
                    "body": body.decode("utf-8", "replace"),
                }
                # A zero content-length is how some clients spell "no body".
                if record["headers"].get("content-length") == "0":
                    del record["headers"]["content-length"]
                response = mock.respond(record)
                if mock.verbose:
                    print(json.dumps(record, ensure_ascii=False), flush=True)
                if response.hang:
                    time.sleep(response.hang)
                    self.close_connection = True
                    return
                self.send_response(response.status)
                if response.content_type:
                    self.send_header("Content-Type", response.content_type)
                if response.chunks is not None:
                    self.send_header("Transfer-Encoding", "chunked")
                    self.end_headers()
                    for chunk in response.chunks:
                        self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
                        self.wfile.flush()
                        time.sleep(0.01)
                    self.wfile.write(b"0\r\n\r\n")
                else:
                    self.send_header("Content-Length", str(len(response.body)))
                    self.end_headers()
                    self.wfile.write(response.body)
                self.wfile.flush()

            do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_HEAD = do_OPTIONS = handle_any

        self.server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
        self.server.daemon_threads = True
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def start(self):
        self.thread.start()
        return self

    def stop(self):
        self.server.shutdown()
        self.server.server_close()

    def configure(self, preset="ok", routes=None):
        """Set the default preset and per-route presets, clear the recording."""
        with self.lock:
            self.preset = preset
            self.routes = {k: [p for p in v.split(" ;; ")] for k, v in (routes or {}).items()}
            self.requests = []

    def take_requests(self):
        with self.lock:
            requests, self.requests = self.requests, []
            return requests

    def respond(self, record):
        with self.lock:
            self.requests.append(record)
            key = f"{record['method']} {record['path']}"
            queue = self.routes.get(key)
            if queue:
                preset = queue.pop(0) if len(queue) > 1 else queue[0]
            else:
                preset = self.preset
        return parse_preset(preset)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--port", type=int, default=8123)
    parser.add_argument("--preset", default="data", help="default response preset")
    parser.add_argument("--route", action="append", default=[], metavar="'METHOD /path=preset'")
    args = parser.parse_args()
    mock = MockSupervisor(args.port, args.preset, verbose=True)
    mock.configure(args.preset, dict(r.split("=", 1) for r in args.route))
    print(f"mock supervisor listening on 127.0.0.1:{mock.port}", file=sys.stderr)
    try:
        mock.server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
