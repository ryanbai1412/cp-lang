#!/usr/bin/env python3
"""Stress a port whose problem accepts more than one answer.

`cp.py stress` compares outputs word for word. This runs the same loop but
asks a checker instead: checker <input> <port's output> <original's output>.

Usage: judge.py port.cp original.cpp checker.cpp [iterations] [size]
"""
import os, subprocess, sys

here = os.path.dirname(os.path.abspath(__file__))
cp_py = os.path.join(here, "..", "..", "..", "cp.py")

def sh(cmd, inp=None, timeout=60):
    return subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=timeout)

def cc(out, src, *flags):
    r = sh(["g++", "-std=c++17", "-O2", *flags, "-o", out, src], timeout=180)
    if r.returncode:
        sys.exit(r.stderr[:3000])
    return out

port, original, checker = sys.argv[1:4]
iters = int(sys.argv[4]) if len(sys.argv) > 4 else 200
size = int(sys.argv[5]) if len(sys.argv) > 5 else 6
base = os.path.splitext(port)[0]
r = sh([sys.executable, cp_py, "build", port])
if r.returncode:
    sys.exit(r.stderr)
ours, gen = cc(base + "_bin", base + ".cpp"), cc(base + "_gen", base + ".cpp", "-DCP_GEN_MODE")
ref, check = cc(base + "_ref", original), cc(base + "_check", checker)
ok = rej = 0
for seed in range(1, iters + 1):
    g = sh([gen, str(seed), str(size)])
    if g.returncode == 4:
        rej += 1
        continue
    a, b = sh([ours], g.stdout), sh([ref], g.stdout)
    files = [base + ext for ext in (".in", ".ours", ".ref")]
    for path, text in zip(files, (g.stdout, a.stdout, b.stdout)):
        open(path, "w").write(text)
    c = sh([check, *files])
    if a.returncode or c.returncode:
        print(f"REJECTED seed={seed}: {c.stdout}\ninput:\n{g.stdout}\nours:\n{a.stdout}{a.stderr}\nref:\n{b.stdout}")
        sys.exit(2)
    ok += 1
print(f"stress OK: {ok} tests accepted ({rej} rejected by where)")
