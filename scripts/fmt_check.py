#!/usr/bin/env python3
"""`cpl fmt` is idempotent and meaning-preserving on every tracked .cp file.

    scripts/fmt_check.py [--cpl PATH] [--jobs N] [--list]

Formats a copy of every tracked `.cp` file (plus `cp.toml`) in a scratch
tree, then checks:

1. idempotence: formatting the formatted tree again changes no file;
2. meaning: for every file the formatter changed, `cpl check` gives the
   same exit status and diagnostic codes on the original and the formatted
   copy, and when the original checks, `cpl run` gives the same exit status,
   standard output and diagnostic codes on its first sample input (the
   `// input:` header, `<stem>.in`, `<stem>.tests/*.in` or `tests/*.in`
   beside it), or on empty input.

A file the lexer rejects is reported by `cpl fmt` and left alone, so it
counts as preserved; a formatter bug (`formatter bug:`) fails the check.
Exit status 1 when any file fails.
"""

import argparse
import concurrent.futures as cf
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CODE = re.compile(r"\b[ERLW]\d{4}\b")
RUN_TIMEOUT = 20


def tracked(pattern_suffixes):
    out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True).stdout
    return sorted(p for p in out.decode().split("\0") if p and p.endswith(pattern_suffixes))


def unescape(s):
    out, i = [], 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s):
            n = s[i + 1]
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def sample_input(path, text):
    for line in text.splitlines()[:6]:
        m = re.match(r'\s*//\s*input:\s*"(.*)"\s*$', line)
        if m:
            return unescape(m.group(1)).encode()
    full = os.path.join(ROOT, path)
    stem = full[: -len(".cp")]
    candidates = [stem + ".in"]
    for d in (stem + ".tests", os.path.join(os.path.dirname(full), "tests")):
        if os.path.isdir(d):
            candidates += sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".in"))
    for c in candidates:
        if os.path.isfile(c):
            with open(c, "rb") as f:
                return f.read()
    return b""


def run(cpl, cwd, args, stdin=b""):
    try:
        p = subprocess.run([cpl] + args, cwd=cwd, input=stdin, capture_output=True, timeout=RUN_TIMEOUT)
    except subprocess.TimeoutExpired:
        return None
    err = p.stderr.decode(errors="replace") + p.stdout.decode(errors="replace") * (args[0] == "check")
    return p.returncode, p.stdout if args[0] == "run" else b"", sorted(CODE.findall(err))


def compare(cpl, scratch, path, original):
    a = run(cpl, ROOT, ["check", path])
    b = run(cpl, scratch, ["check", path])
    if a is None:
        return None
    if a != b:
        return f"cpl check differs: original {a[0]} {a[2][:5]}, formatted {b and (b[0], b[2][:5])}"
    if a[0] != 0:
        return None
    stdin = sample_input(path, original)
    a = run(cpl, ROOT, ["run", path], stdin)
    if a is None:
        return None
    b = run(cpl, scratch, ["run", path], stdin)
    if a != b:
        if b is None:
            return "cpl run times out after formatting"
        what = "exit status" if a[0] != b[0] else "output" if a[1] != b[1] else "diagnostics"
        return f"cpl run {what} differs: original {a[0]} {a[2][:5]}, formatted {b[0]} {b[2][:5]}"
    return None


def fmt_tree(cpl, tree):
    p = subprocess.run([cpl, "fmt", "."], cwd=tree, capture_output=True, text=True)
    return [line.split(": not formatted: ")[0] for line in p.stderr.splitlines() if "not formatted: formatter bug" in line]


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--cpl", default=os.path.join(os.environ.get("CARGO_TARGET_DIR", os.path.join(ROOT, "target")), "release", "cpl"))
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--list", action="store_true", help="list every failing file, not just the first 20")
    args = ap.parse_args()
    cpl = os.path.abspath(args.cpl)
    files = tracked((".cp",))
    with tempfile.TemporaryDirectory(prefix="fmt-check-") as tmp:
        once, twice = os.path.join(tmp, "once"), os.path.join(tmp, "twice")
        for path in files + tracked(("cp.toml",)):
            dst = os.path.join(once, path)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copyfile(os.path.join(ROOT, path), dst)
        bugs = fmt_tree(cpl, once)
        shutil.copytree(once, twice)
        fmt_tree(cpl, twice)
        originals, formatted, changed, unstable = {}, {}, [], []
        for path in files:
            with open(os.path.join(ROOT, path), encoding="utf-8", errors="surrogateescape") as f:
                originals[path] = f.read()
            with open(os.path.join(once, path), encoding="utf-8", errors="surrogateescape") as f:
                formatted[path] = f.read()
            with open(os.path.join(twice, path), encoding="utf-8", errors="surrogateescape") as f:
                again = f.read()
            if formatted[path] != originals[path]:
                changed.append(path)
            if again != formatted[path]:
                unstable.append(path)
        with cf.ThreadPoolExecutor(args.jobs) as pool:
            verdicts = dict(zip(changed, pool.map(lambda p: compare(cpl, once, p, originals[p]), changed)))
    altered = {p: v for p, v in verdicts.items() if v}
    print(f"fmt-check: {len(files)} files, {len(changed)} changed by cpl fmt, "
          f"{len(unstable)} not idempotent, {len(altered)} change meaning, {len(bugs)} formatter bugs")
    limit = None if args.list else 20
    for title, items in (("not idempotent (fmt(fmt(x)) != fmt(x))", [(p, "") for p in unstable]),
                         ("meaning changed", sorted(altered.items())),
                         ("formatter bug", [(p, "") for p in bugs])):
        if items:
            print(f"\n{title}:")
            for p, why in items[:limit]:
                print(f"  {p}" + (f": {why}" if why else ""))
            if limit and len(items) > limit:
                print(f"  ... and {len(items) - limit} more (--list shows all)")
    return 1 if unstable or altered or bugs else 0


if __name__ == "__main__":
    sys.exit(main())
