#!/usr/bin/env python3
"""Command-tree diff: walk the help of the Go CLI and the Rust CLI
recursively and compare, for every command, the sub commands, aliases,
flags (name, shorthand, value type, default, description), usage line,
long/short description and examples.

    python3 tests/tree_diff.py              # whole tree
    python3 tests/tree_diff.py dns          # only `ha dns ...`
    python3 tests/tree_diff.py os boards    # only `ha os boards ...`
    python3 tests/tree_diff.py --loose dns  # ignore pure help text differences
                                            # (descriptions, examples, flag usage texts)

Because the Rust CLI renders help in cobra's exact format, the default mode
requires the help output to be byte-identical; on a difference the report
names what differs structurally (commands / aliases / flags / usage) and
shows a unified diff of the help text. For each command the flag-name
completion (`ha __complete <path> -`) is compared as well.

Hidden commands and hidden/deprecated flags do not appear in help output and
are therefore not covered here; cover them with parity cases.
"""

import argparse
import difflib
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import parity  # noqa: E402

FLAG = re.compile(r"^\s+(?:-(?P<short>\S), )?--(?P<name>[^\s\[]+)(?P<noopt>\[=[^\]]*\])?(?: (?P<type>\S+))?\s{2,}(?P<usage>.*)$")
ENV = {
    "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
    "HOME": "/nonexistent",
    # Nothing listens here: dynamic completers fail fast and identically.
    "SUPERVISOR_ENDPOINT": "127.0.0.1:1",
}


# Prefix of differences that concern help texts only (ignored by --loose).
TEXT_ONLY = "text: "


def run(binary, args):
    proc = subprocess.run([binary] + args, capture_output=True, env=ENV, timeout=30)
    return proc.stdout.decode("utf-8", "replace") + proc.stderr.decode("utf-8", "replace"), proc.returncode


def parse_help(text):
    """Split cobra help output into its parts."""
    info = {"description": "", "usage": [], "aliases": [], "examples": "", "commands": {}, "flags": {}, "global": {}}
    head, sep, rest = text.partition("Usage:\n")
    if not sep:
        info["description"] = text
        return info
    info["description"] = head
    section = "usage"
    for line in rest.splitlines():
        if line in ("Aliases:", "Examples:", "Available Commands:", "Flags:", "Global Flags:", "Additional help topics:"):
            section = line[:-1].lower()
            continue
        if line.startswith("Use \"") and line.endswith("for more information about a command."):
            continue
        if section == "usage" and line.strip():
            info["usage"].append(line.strip())
        elif section == "aliases" and line.strip():
            info["aliases"] = [a.strip() for a in line.split(",")]
        elif section == "examples":
            info["examples"] += line + "\n"
        elif section in ("available commands", "additional help topics") and line.strip():
            name, _, short = line.strip().partition(" ")
            info["commands"][name] = short.strip()
        elif section in ("flags", "global flags"):
            m = FLAG.match(line)
            if m:
                usage = m.group("usage")
                default = re.search(r"\(default (.*)\)$", usage)
                info["flags" if section == "flags" else "global"][m.group("name")] = {
                    "shorthand": m.group("short") or "",
                    "type": m.group("type") or "bool",
                    "no-opt-value": m.group("noopt") or "",
                    "default": default.group(1) if default else "",
                    "usage": usage,
                }
    return info


def describe_differences(go, rust):
    """Human readable structural differences between two parsed helps."""
    out = []
    for key, label in (("usage", "usage line"), ("aliases", "aliases")):
        if go[key] != rust[key]:
            out.append(f"{label}: go={go[key]} rust={rust[key]}")
    for name in sorted(set(go["commands"]) | set(rust["commands"])):
        if name not in rust["commands"]:
            out.append(f"sub command missing in rust: {name}")
        elif name not in go["commands"]:
            out.append(f"sub command only in rust: {name}")
        elif go["commands"][name] != rust["commands"][name]:
            out.append(f"text: sub command {name}: short description differs")
    for kind in ("flags", "global"):
        for name in sorted(set(go[kind]) | set(rust[kind])):
            if name not in rust[kind]:
                out.append(f"{kind} flag missing in rust: --{name}")
            elif name not in go[kind]:
                out.append(f"{kind} flag only in rust: --{name}")
            else:
                for attr in ("shorthand", "type", "no-opt-value", "default", "usage"):
                    a, b = go[kind][name][attr], rust[kind][name][attr]
                    if a != b:
                        prefix = "text: " if attr == "usage" else ""
                        out.append(f"{prefix}{kind} flag --{name}: {attr} differs: go={a!r} rust={b!r}")
    if go["description"] != rust["description"]:
        out.append("text: long/short description differs")
    if go["examples"] != rust["examples"]:
        out.append("text: examples differ")
    return out


def main():
    parser = argparse.ArgumentParser(description="Go vs Rust `ha` command tree diff")
    parser.add_argument("path", nargs="*", help="command path to start from (default: the root)")
    parser.add_argument("--loose", action="store_true", help="only compare structure, not the exact help text")
    parser.add_argument("--no-build", action="store_true", help="do not run `cargo build` first")
    args = parser.parse_args()

    go, rust = parity.go_binary(), parity.rust_binary(not args.no_build)
    queue = [list(args.path)]
    checked = failed = 0
    while queue:
        path = queue.pop(0)
        label = " ".join(["ha"] + path)
        go_help, go_code = run(go, path + ["--help"])
        rust_help, rust_code = run(rust, path + ["--help"])
        go_info, rust_info = parse_help(go_help), parse_help(rust_help)
        problems = describe_differences(go_info, rust_info)
        if go_code != rust_code:
            problems.append(f"--help exit code: go={go_code} rust={rust_code}")
        text_differs = go_help != rust_help
        if args.loose:
            problems = [p for p in problems if not p.startswith(TEXT_ONLY)]
            text_differs = False
        go_comp, _ = run(go, ["__complete"] + path + ["-"])
        rust_comp, _ = run(rust, ["__complete"] + path + ["-"])
        if go_comp != rust_comp:
            problems.append(("text: " if args.loose else "") + "flag completion (`__complete ... -`) differs")
            problems = [p for p in problems if not (args.loose and p.startswith(TEXT_ONLY))]
        checked += 1
        if problems or text_differs:
            failed += 1
            print(f"DIFF {label}")
            for problem in problems:
                print(f"     - {problem}")
            if not args.loose:
                diff = difflib.unified_diff(
                    go_help.splitlines(True), rust_help.splitlines(True), "go --help", "rust --help"
                )
                sys.stdout.writelines("     " + line for line in diff)
                if go_comp != rust_comp:
                    diff = difflib.unified_diff(
                        go_comp.splitlines(True), rust_comp.splitlines(True), "go __complete -", "rust __complete -"
                    )
                    sys.stdout.writelines("     " + line for line in diff)
        else:
            print(f"ok   {label}")
        # Recurse into the sub commands either binary knows about.
        children = sorted(set(go_info["commands"]) | set(rust_info["commands"]))
        for child in children:
            if child == "help":
                continue
            queue.append(path + [child])
    print(f"\n{checked} commands compared, {failed} differ")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
