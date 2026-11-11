#!/usr/bin/env python3
"""Parity runner: run the Go CLI and the Rust CLI with identical command
lines against a mock Supervisor and diff stdout, stderr, exit code and the
HTTP requests they made.

    python3 tests/parity.py                 # all groups
    python3 tests/parity.py dns             # only tests/parity/cases/dns.txt
    python3 tests/parity.py dns -k logs     # cases whose name contains "logs"
    python3 tests/parity.py dns --show      # also print what the Go CLI did

Case files live in tests/parity/cases/<group>.txt, one file per command
group.  Format (see tests/parity/cases/dns.txt):

    # comment
    == dns info                       <- starts a case; this is its name and,
                                         unless "args:" is given, its command line
    response: data                    <- default response preset (default: ok)
    route: GET /dns/info = error:400  <- per-route preset(s), "a ;; b" = sequence
    args: dns info --raw-json         <- command line (shell quoting; "" = empty arg)
    stdin: YES\\n                      <- piped to stdin (\\n, \\t escapes); default: empty
    env: SUPERVISOR_RAW_JSON=true     <- extra environment variable
    unset: SUPERVISOR_TOKEN           <- remove a variable from the environment
    file: cfg.yaml = raw-json: true\\n <- create {tmp}/cfg.yaml before running
    compare: stdout exit requests     <- restrict what is compared
                                         (default: stdout stderr exit requests)

Placeholders usable in args/env/file/stdin: {endpoint} (host:port of the
mock), {tmp} (fresh per-case directory), {home} (the empty HOME directory).
Response presets are documented in tests/mock_supervisor.py.

Both binaries run with a minimal environment: PATH, HOME={home},
SUPERVISOR_ENDPOINT={endpoint}, SUPERVISOR_TOKEN=parity-token.

The Go reference binary is built from the Go source tree into
target/go/ha (outside the Go repository) when `go` is available; otherwise
the prebuilt binary in the Go repository is used.
"""

import argparse
import difflib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)

from mock_supervisor import MockSupervisor  # noqa: E402

GO_REPO = os.environ.get("HA_GO_REPO", os.path.join(os.path.dirname(ROOT), "cli"))
GO_BIN = os.environ.get("HA_GO_BIN", os.path.join(ROOT, "target", "go", "ha"))
RUST_BIN = os.environ.get("HA_RUST_BIN", os.path.join(ROOT, "target", "debug", "ha"))
CASES_DIR = os.path.join(HERE, "parity", "cases")
ALL_FIELDS = ("stdout", "stderr", "exit", "requests")
TIMEOUT = 30


def go_binary(rebuild=False):
    """Return the path of the Go reference binary, building it if possible."""
    newest_source = 0
    for base, _dirs, files in os.walk(GO_REPO):
        for name in files:
            if name.endswith(".go") or name in ("go.mod", "go.sum"):
                newest_source = max(newest_source, os.path.getmtime(os.path.join(base, name)))
    stale = not os.path.exists(GO_BIN) or os.path.getmtime(GO_BIN) < newest_source
    if (stale or rebuild) and shutil.which("go"):
        os.makedirs(os.path.dirname(GO_BIN), exist_ok=True)
        print(f"building Go reference binary -> {GO_BIN}", file=sys.stderr)
        env = dict(os.environ, CGO_ENABLED="0")
        subprocess.run(["go", "build", "-o", GO_BIN, "."], cwd=GO_REPO, env=env, check=True)
    if os.path.exists(GO_BIN):
        return GO_BIN
    prebuilt = os.path.join(GO_REPO, "ha")
    if os.path.exists(prebuilt):
        print(f"NOTE: `go` not available, using prebuilt Go binary {prebuilt}", file=sys.stderr)
        return prebuilt
    sys.exit("no Go reference binary: install go or set HA_GO_BIN")


def rust_binary(build=True):
    if build and "HA_RUST_BIN" not in os.environ:
        subprocess.run(["cargo", "build", "--quiet"], cwd=ROOT, check=True)
    if not os.path.exists(RUST_BIN):
        sys.exit(f"Rust binary not found: {RUST_BIN}")
    return RUST_BIN


class Case:
    def __init__(self, name, source):
        self.name = name
        self.source = source
        self.args = None
        self.response = "ok"
        self.routes = {}
        self.stdin = ""
        self.env = {}
        self.unset = []
        self.files = {}
        self.compare = ALL_FIELDS


def parse_cases(path):
    cases = []
    case = None
    with open(path, encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            line = line.rstrip("\n")
            where = f"{os.path.basename(path)}:{lineno}"
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            if line.startswith("=="):
                case = Case(line[2:].strip(), where)
                cases.append(case)
                continue
            if case is None or ":" not in line:
                sys.exit(f"{where}: expected '== case name' or 'key: value'")
            key, value = line.split(":", 1)
            key, value = key.strip(), value.strip()
            if key == "args":
                case.args = value
            elif key == "response":
                case.response = value
            elif key == "route":
                route, preset = value.split("=", 1)
                case.routes[" ".join(route.split())] = preset.strip()
            elif key == "stdin":
                case.stdin = value
            elif key == "env":
                name, val = value.split("=", 1)
                case.env[name.strip()] = val
            elif key == "unset":
                case.unset.append(value)
            elif key == "file":
                name, content = value.split("=", 1)
                case.files[name.strip()] = content.lstrip(" ")
            elif key == "compare":
                case.compare = tuple(value.split())
                unknown = set(case.compare) - set(ALL_FIELDS)
                if unknown:
                    sys.exit(f"{where}: unknown compare field(s) {sorted(unknown)}")
            else:
                sys.exit(f"{where}: unknown key {key!r}")
    for case in cases:
        if case.args is None:
            case.args = case.name
    return cases


def unescape(text):
    return text.replace("\\n", "\n").replace("\\t", "\t")


TIMESTAMP = re.compile(r"^\d{4}/\d\d/\d\d \d\d:\d\d:\d\d ", re.M)


def run_binary(binary, case, mock, home):
    """Run one binary for a case; return the observable behaviour."""
    with tempfile.TemporaryDirectory(prefix="ha-parity-") as tmp:
        endpoint = f"127.0.0.1:{mock.port}"

        def subst(text):
            return text.replace("{endpoint}", endpoint).replace("{tmp}", tmp).replace("{home}", home)

        for name, content in case.files.items():
            target = os.path.join(tmp, name)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "w", encoding="utf-8") as fh:
                fh.write(unescape(subst(content)))
        env = {
            "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
            "HOME": home,
            "SUPERVISOR_ENDPOINT": endpoint,
            "SUPERVISOR_TOKEN": "parity-token",
        }
        env.update({k: subst(v) for k, v in case.env.items()})
        for name in case.unset:
            env.pop(name, None)
        argv = [binary] + shlex.split(subst(case.args))
        mock.configure(case.response, case.routes)
        try:
            proc = subprocess.run(
                argv,
                input=unescape(subst(case.stdin)).encode(),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
                cwd=tmp,
                timeout=TIMEOUT,
            )
            stdout, stderr, code = proc.stdout, proc.stderr, proc.returncode
        except subprocess.TimeoutExpired as exc:
            stdout, stderr, code = exc.stdout or b"", exc.stderr or b"", f"timeout after {TIMEOUT}s"
        requests = mock.take_requests()

        def clean(data):
            text = data.decode("utf-8", "replace").replace(tmp, "{tmp}").replace(endpoint, "{endpoint}")
            return TIMESTAMP.sub("<timestamp> ", text)

        return {
            "stdout": clean(stdout),
            "stderr": clean(stderr),
            "exit": str(code) + "\n",
            "requests": "".join(json.dumps(r, sort_keys=True, ensure_ascii=False) + "\n" for r in requests),
        }


def diff(field, go, rust):
    lines = difflib.unified_diff(
        go.splitlines(keepends=True), rust.splitlines(keepends=True), f"go {field}", f"rust {field}"
    )
    out = []
    for line in lines:
        out.append(line if line.endswith("\n") else line + "\n\\ No newline at end\n")
    return "".join(out)


def main():
    parser = argparse.ArgumentParser(description="Go vs Rust `ha` parity runner")
    parser.add_argument("groups", nargs="*", help="case files to run (default: all)")
    parser.add_argument("-k", dest="filter", default="", help="only cases whose name contains this")
    parser.add_argument("--show", action="store_true", help="print the Go CLI behaviour of every case")
    parser.add_argument("--no-build", action="store_true", help="do not run `cargo build` first")
    parser.add_argument("--rebuild-go", action="store_true", help="force a rebuild of the Go reference binary")
    parser.add_argument("-x", "--exitfirst", action="store_true", help="stop at the first failing case")
    args = parser.parse_args()

    go = go_binary(args.rebuild_go)
    rust = rust_binary(not args.no_build)
    groups = args.groups or sorted(f[:-4] for f in os.listdir(CASES_DIR) if f.endswith(".txt"))

    mock = MockSupervisor().start()
    home = tempfile.mkdtemp(prefix="ha-parity-home-")
    passed = failed = 0
    failures = []
    try:
        for group in groups:
            path = os.path.join(CASES_DIR, group + ".txt")
            if not os.path.exists(path):
                sys.exit(f"no such case file: {path}")
            for case in parse_cases(path):
                if args.filter not in case.name:
                    continue
                go_result = run_binary(go, case, mock, home)
                rust_result = run_binary(rust, case, mock, home)
                diffs = [
                    diff(field, go_result[field], rust_result[field])
                    for field in case.compare
                    if go_result[field] != rust_result[field]
                ]
                label = f"{group}: {case.name}"
                if diffs:
                    failed += 1
                    failures.append(label)
                    print(f"FAIL {label}  ({case.source})\n     args: {case.args}")
                    print("".join(diffs))
                else:
                    passed += 1
                    print(f"ok   {label}")
                if args.show:
                    for field in ALL_FIELDS:
                        body = go_result[field]
                        print(f"  --- go {field} ---\n" + "".join("  | " + line for line in body.splitlines(True)))
                        if body and not body.endswith("\n"):
                            print()
                if diffs and args.exitfirst:
                    raise KeyboardInterrupt
    except KeyboardInterrupt:
        pass
    finally:
        mock.stop()
        shutil.rmtree(home, ignore_errors=True)
    print(f"\n{passed} passed, {failed} failed (go: {go}, rust: {rust})")
    if passed + failed == 0:
        sys.exit("no cases matched")
    for label in failures:
        print(f"  FAILED {label}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
