#!/usr/bin/env python3
"""Checker for CF 2245 C: the same YES/NO verdicts as the original, and every
printed p is a permutation of 0..n-1 whose prefix mexes XOR to k."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = open(sys.argv[1], encoding="utf-8").read().split()
    out = open(sys.argv[2], encoding="utf-8").read().split()
    ref = open(sys.argv[3], encoding="utf-8").read().split()
    t = int(inp[0])
    pi, po, pr = 1, 0, 0
    for case in range(1, t + 1):
        n, k = int(inp[pi]), int(inp[pi + 1])
        pi += 2
        if po >= len(out):
            fail(f"case {case}: output ends early")
        got, want = out[po].upper(), ref[pr].upper()
        po += 1
        pr += 1 + (n if want == "YES" else 0)
        if got not in ("YES", "NO"):
            fail(f"case {case}: expected YES or NO, got {out[po - 1]!r}")
        if got != want:
            fail(f"case {case}: verdict {got}, original says {want}")
        if got == "NO":
            continue
        p = [int(x) for x in out[po:po + n]]
        po += n
        if len(p) != n or sorted(p) != list(range(n)):
            fail(f"case {case}: not a permutation of 0..{n - 1}")
        seen = [False] * (n + 1)
        mex = acc = 0
        for v in p:
            seen[v] = True
            while seen[mex]:
                mex += 1
            acc ^= mex
        if acc != k:
            fail(f"case {case}: prefix mexes XOR to {acc}, want {k}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
