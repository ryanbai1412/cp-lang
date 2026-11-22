#!/usr/bin/env python3
"""Stress a port's main solution against its own in-file `@brute(main) fn slow()`.

usage: selfstress.py <port.cp> [seeds per size=200] [max size=8] [brute name=slow]
Sizes go 1, 2, ... so the first mismatch printed is also the smallest one found.
The port is copied to a scratch directory first, so the tracked file is not stamped.
"""
import os, shutil, subprocess, sys, tempfile
here = os.path.dirname(os.path.abspath(__file__))
cp = os.path.join(here, "..", "..", "cp.py")
src = os.path.abspath(sys.argv[1])
seeds = int(sys.argv[2]) if len(sys.argv) > 2 else 200
max_size = int(sys.argv[3]) if len(sys.argv) > 3 else 8
brute = sys.argv[4] if len(sys.argv) > 4 else "slow"
tmp = tempfile.mkdtemp()
base = os.path.join(tmp, os.path.basename(src)[:-3])
shutil.copy(src, base + ".cp")
r = subprocess.run([sys.executable, cp, "build", base + ".cp"], capture_output=True, text=True)
if r.returncode:
    sys.exit(r.stderr)
sys.stderr.write("".join(l + "\n" for l in r.stderr.splitlines() if "warning" in l))
for out, flags in ((base + "_bin", []), (base + "_gen", ["-DCP_GEN_MODE"])):
    r = subprocess.run(["g++", "-std=c++17", "-O2", *flags, "-o", out, base + ".cpp"], capture_output=True, text=True)
    if r.returncode:
        sys.exit(r.stderr[:3000])
def run(inp, sol=None):
    env = dict(os.environ)
    if sol: env["CP_SOLUTION"] = sol
    p = subprocess.run([base + "_bin"], input=inp, capture_output=True, text=True, timeout=20, env=env)
    return p.returncode, p.stdout, p.stderr
ok = rej = 0
for size in range(1, max_size + 1):
    for seed in range(1, seeds + 1):
        g = subprocess.run([base + "_gen", str(seed), str(size)], capture_output=True, text=True, timeout=20)
        if g.returncode:
            rej += 1
            continue
        a, b = run(g.stdout), run(g.stdout, brute)
        if a[0] or b[0] or a[1].split() != b[1].split():
            print(f"MISMATCH size={size} seed={seed} (after {ok} passing)\ninput:\n{g.stdout}\nmain (exit {a[0]}):\n{a[1]}{a[2]}\n{brute} (exit {b[0]}):\n{b[1]}{b[2]}")
            sys.exit(2)
        ok += 1
print(f"selfstress OK: {ok} tests agree ({rej} rejected by where)")
