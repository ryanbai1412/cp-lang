#!/usr/bin/env python3
"""Stage cache for scripts/check.sh --fast.

A stage's key hashes the whole working tree (every file git tracks or would
track) minus the stage's explicit exclude patterns, plus the toolchain. A
stage is skipped only when its key matches the one saved after it last
passed. Forgetting to exclude a path only costs a rerun; excluding a path the
stage reads would skip a needed run, so each exclude needs evidence that the
stage never opens it (STAGES below says where that evidence came from).

    check_cache.py manifest <file>                 hash every file once
    check_cache.py <check|save|key> <cache_dir> <stage> <manifest>

A pattern is an fnmatch glob over repository paths (`*` crosses `/`); a
pattern starting with `!` keeps paths an earlier pattern excluded.
"""

import fnmatch
import hashlib
import os
import subprocess
import sys

TOOLS = (
    ["rustc", "-vV"],
    ["cargo", "-V"],
    ["g++", "--version"],
    ["python3", "--version"],
    ["node", "--version"],
)
ENV_PREFIXES = ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_", "CPL_", "CHECK_", "CXX", "CC")

# Every exclude was checked against an strace of the stage on a cold target
# (open/stat/exec of every path, followed into child processes): the stage
# opened none of the excluded paths. Re-run that before adding one; grep is
# not enough (the tests open design/spec/ through cpl-syntax's planned.rs).
_PROSE = ["AGENTS.md", "REPO.md", "decisions.md", "research/*"]
_TESTS = _PROSE + [
    "!research/scoreboard.json",
    "corpus/submissions/*", "corpus/statements/*", "plan/*",
]
STAGES = {
    # rustc reads only crates/, spec/std/ (cpl-check's build.rs) and the
    # include_str! targets; test_check_cache.py keeps those out of here.
    "clippy": _TESTS + [
        "README.md", "bench/*", "docs/*", "editors/*", "examples/*", "web/*",
        "corpus/*", "tools/*",
        "!corpus/problems/cf2115c/dream.cp", "!corpus/problems/cf2183f/dream.cp",
        "!tools/trace/*",
    ],
    "stdlib_test": _TESTS,
    "nextest_doc": _TESTS,
    "recovery": _TESTS,
    # The public-history scan reads every tracked file.
    "scoreboard_fast": [],
    "site_gate": ["archive/*"],
}


def tree_files(root):
    out = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root, capture_output=True, check=True,
    ).stdout
    return sorted(set(p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p))


def file_digest(path):
    if os.path.islink(path):
        return "L" + hashlib.sha256(os.readlink(path).encode()).hexdigest()
    if not os.path.isfile(path):
        return "MISSING"
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    mode = "X" if os.access(path, os.X_OK) else "F"
    return mode + h.hexdigest()


def environment():
    h = hashlib.sha256()
    for cmd in TOOLS:
        try:
            r = subprocess.run(cmd, capture_output=True, timeout=30)
            h.update(r.stdout + r.stderr)
        except (OSError, subprocess.TimeoutExpired):
            h.update(b"absent")
        h.update(b"\0")
    for k in sorted(os.environ):
        if k.startswith(ENV_PREFIXES):
            h.update(f"{k}={os.environ[k]}\0".encode())
    return h.hexdigest()


def manifest(root):
    """`<env digest>` then one `<digest> <path>` line per file in the tree."""
    lines = [environment()]
    for rel in tree_files(root):
        lines.append(f"{file_digest(os.path.join(root, rel))} {rel}")
    return "\n".join(lines) + "\n"


def excluded(path, patterns):
    out = False
    for p in patterns:
        if p.startswith("!"):
            if out and fnmatch.fnmatchcase(path, p[1:]):
                out = False
        elif not out and fnmatch.fnmatchcase(path, p):
            out = True
    return out


def stage_key(manifest_text, stage, patterns):
    env, *entries = manifest_text.splitlines()
    h = hashlib.sha256()
    h.update(f"stage {stage}\0env {env}\0".encode())
    for p in patterns:
        h.update(f"pattern {p}\0".encode())
    for line in entries:
        _, _, path = line.partition(" ")
        if not excluded(path, patterns):
            h.update(line.encode("utf-8", "surrogateescape") + b"\0")
    return h.hexdigest()


def cache_file(cache_dir, stage):
    return os.path.join(cache_dir, f"{stage}.sha256")


def is_hit(cache_dir, stage, key):
    try:
        with open(cache_file(cache_dir, stage), encoding="utf-8") as f:
            return f.read().strip() == key
    except OSError:
        return False


def save_hit(cache_dir, stage, key):
    os.makedirs(cache_dir, exist_ok=True)
    tmp = cache_file(cache_dir, stage) + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(key + "\n")
    os.replace(tmp, cache_file(cache_dir, stage))


def main(argv=None):
    args = list(sys.argv[1:] if argv is None else argv)
    if len(args) == 2 and args[0] == "manifest":
        text = manifest(os.getcwd())
        tmp = args[1] + ".tmp"
        with open(tmp, "w", encoding="utf-8", errors="surrogateescape") as f:
            f.write(text)
        os.replace(tmp, args[1])
        return 0
    if len(args) != 4 or args[0] not in ("check", "save", "key") or args[2] not in STAGES:
        print(__doc__, file=sys.stderr)
        return 2
    cmd, cache_dir, stage, manifest_path = args
    with open(manifest_path, encoding="utf-8", errors="surrogateescape") as f:
        key = stage_key(f.read(), stage, STAGES[stage])
    if cmd == "key":
        print(key)
        return 0
    if cmd == "check":
        return 0 if is_hit(cache_dir, stage, key) else 1
    save_hit(cache_dir, stage, key)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
