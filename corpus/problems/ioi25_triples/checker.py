#!/usr/bin/env python3
"""Checker for IOI 2025 Triple Peaks (the official checker/checker.cpp's rules). Part I: the
count T equals the answer. Part II: 3 <= N <= M heights in 1..=N-1 forming at least K
mythical triples, counted by the accepted C++ (orig.cpp) on the range as a Part I input."""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "scripts"))


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = open(sys.argv[1]).read().split()
    out = open(sys.argv[2]).read().split()
    if inp[0] == "1":
        want = open(sys.argv[3]).read().split()
        if out != want:
            fail(f"got {' '.join(out[:1])}, want {' '.join(want[:1])}")
        return
    m, k = int(inp[1]), int(inp[2])
    if not out or not out[0].lstrip("-").isdigit():
        fail("N is not a number")
    n = int(out[0])
    if not 3 <= n <= m:
        fail(f"N = {n} is not in 3..={m}")
    h = out[1:]
    if len(h) != n:
        fail(f"length of H is {len(h)}, not N = {n}")
    if not all(x.isdigit() and 1 <= int(x) <= n - 1 for x in h):
        fail(f"a height is not in 1..={n - 1}")
    import corpus
    exe = corpus.binary(os.path.join(HERE, "orig.cpp"))
    r = subprocess.run([exe], input=f"1\n{n}\n{' '.join(h)}\n", capture_output=True, text=True, check=True)
    t = int(r.stdout.split()[0])
    if t < k:
        fail(f"T = {t} mythical triples, want at least K = {k}")


if __name__ == "__main__":
    main()
