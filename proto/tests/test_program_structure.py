#!/usr/bin/env python3
"""Program structure (decision 6.3): the top-level `@!multitest` divider and top-level `return`.
Each tests/program/x.cp must turn x.in into x.out both built by cpc (under ASan + UBSan) and run by cpi.
Run from proto/."""
import glob, os, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(HERE)
CASES = os.path.join(HERE, "program")
CP = [sys.executable, os.path.join(PROTO, "cp.py")]
CXX = ["g++", "-std=c++17", "-O1", "-DLOCAL", "-fsanitize=address,undefined", "-fno-sanitize-recover=undefined"]

def compiled(src, inp):
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copy(src, os.path.join(tmp, "t.cp"))     # a copy: the build stamps its source
        r = subprocess.run(CP + ["build", os.path.join(tmp, "t.cp")], capture_output=True, text=True)
        if r.returncode:
            return "build failed: " + r.stderr[-300:]
        r = subprocess.run(CXX + ["-o", os.path.join(tmp, "t"), os.path.join(tmp, "t.cpp")], capture_output=True, text=True)
        if r.returncode:
            return "g++ failed: " + r.stderr[:300]
        r = subprocess.run([os.path.join(tmp, "t")], input=inp, capture_output=True, text=True, timeout=20)
        return r.stdout + r.stderr

def interpreted(src, inp):
    r = subprocess.run(CP + ["lib-run", src], input=inp, capture_output=True, text=True, timeout=20, cwd=PROTO)
    return r.stdout + r.stderr

failed = 0
cases = sorted(glob.glob(os.path.join(CASES, "*.cp")))
for src in cases:
    name = os.path.basename(src)[:-3]
    inp, want = (open(os.path.join(CASES, name + ext)).read() for ext in (".in", ".out"))
    for how, run in (("cpc", compiled), ("cpi", interpreted)):
        got = run(src, inp)
        if got != want:
            failed += 1
            print(f"PROGRAM STRUCTURE FAILED: {name} ({how}): wanted {want!r}, got {got[:300]!r}")
print(f"program structure: {2 * len(cases) - failed}/{2 * len(cases)} passed")
sys.exit(1 if failed else 0)
