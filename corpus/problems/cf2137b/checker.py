#!/usr/bin/env python3
"""Checker for CF 2137 B: any permutation q with gcd(p_i + q_i, p_{i+1} + q_{i+1}) >= 3."""
from math import gcd
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def main():
    inp, out, _ = load()
    t, pi, po = int(inp[0]), 1, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        p = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        q = [int(x) for x in out[po:po + n]]
        po += n
        if len(q) != n or sorted(q) != list(range(1, n + 1)):
            fail(f"case {case}: not a permutation of 1..{n}")
        for i in range(n - 1):
            if gcd(p[i] + q[i], p[i + 1] + q[i + 1]) < 3:
                fail(f"case {case}: gcd at positions {i + 1}, {i + 2} is below 3")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
