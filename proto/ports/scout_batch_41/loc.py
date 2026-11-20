#!/usr/bin/env python3
"""Dual LOC for scout batch 41.

A line counts when it is neither blank nor a pure `//` comment.
  raw       the whole original C++ or Java file
  stripped  the original without its contest template:
            - C++: drops everything up to `#define sz(x)` (includes, debug
              macros, aliases), stream-setup lines, and `io::` fast-reader
              boilerplate (`nccc9s4`)
            - Java: drops imports, `static class Input`, `main` boilerplate,
              and class/method wrappers (`nccc8s1`, `nccc8s2`, `nccc8s4`)
  v2        <problem>.cp, the locked v2-spec port
  today     today/<problem>.cp, the runnable form today
"""
import os, re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "oly20practice139": "6415690.cpp",
    "oly23practice5":   "6415646.cpp",
    "nccc9s4":          "4145494.cpp",
    "nccc9s1":          "4051682.cpp",
    "nccc9s2":          "4051671.cpp",
    "nccc9s3":          "4051697.cpp",
    "nccc10s1":         "4214147.cpp",
    "nccc8s1":          "3381111.java",
    "nccc8s2":          "3381135.java",
    "nccc8s4":          "3381215.java",
}
IO_SETUP = re.compile(r"sync_with_stdio|cin\.tie|cin\.exceptions|freopen|#ifdef LOCAL|#endif|io::init")


def code(lines):
    return [l for l in lines if l.strip() and not l.strip().startswith("//")]


def read(path):
    return open(path, encoding="utf-8", errors="replace").read().splitlines()


def strip_orig(name, lines):
    if ORIG[name].endswith(".java"):
        # Drop imports, fast Input class, and main() wrapper; keep fields/methods/solve body
        cut = max(i for i, l in enumerate(lines) if "pw.close()" in l) + 2
        body = lines[cut:]
        # Drop trailing class closing brace
        while body and body[-1].strip() in ("", "}"):
            if body[-1].strip() == "}":
                body.pop()
                break
            body.pop()
        return code(body)
    cut = max(i for i, l in enumerate(lines) if l.startswith("#define sz(")) + 1
    rest = lines[cut:]
    # For nccc9s4, also drop the custom fread namespace io { ... }
    if name == "nccc9s4":
        out = []
        in_io = False
        for l in rest:
            if l.strip().startswith("namespace io {"):
                in_io = True
                continue
            if in_io:
                if l.strip().startswith("}") and "namespace io" in l:
                    in_io = False
                continue
            out.append(l)
        rest = out
    return [l for l in code(rest) if not IO_SETUP.search(l)]


tot = [0, 0, 0, 0]
print(f"{'problem':18} {'raw':>5} {'stripped':>8} {'v2':>4} {'today':>5}")
for name, fname in ORIG.items():
    src = read(os.path.join(ROOT, "corpus", "submissions", fname))
    row = [
        len(code(src)),
        len(strip_orig(name, src)),
        len(code(read(os.path.join(HERE, name + ".cp")))),
        len(code(read(os.path.join(HERE, "today", name + ".cp")))),
    ]
    tot = [a + b for a, b in zip(tot, row)]
    print(f"{name:18} {row[0]:5} {row[1]:8} {row[2]:4} {row[3]:5}")
print(f"{'total':18} {tot[0]:5} {tot[1]:8} {tot[2]:4} {tot[3]:5}")
print(f"v2 vs raw:    -{100 - 100 * tot[2] / tot[0]:.0f}%   v2 vs stripped:    -{100 - 100 * tot[2] / tot[1]:.0f}%")
print(f"today vs raw: -{100 - 100 * tot[3] / tot[0]:.0f}%   today vs stripped: -{100 - 100 * tot[3] / tot[1]:.0f}%")
