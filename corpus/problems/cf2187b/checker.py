#!/usr/bin/env python3
"""Checker for CF 2187 B: any p, q >= 0 with p & q == 0 whose |x - p| + |y - q|
equals the reference answer's cost."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = [int(x) for x in open(sys.argv[1], encoding="utf-8").read().split()]
    out = [int(x) for x in open(sys.argv[2], encoding="utf-8").read().split()]
    ref = [int(x) for x in open(sys.argv[3], encoding="utf-8").read().split()]
    t = inp[0]
    if len(out) != 2 * t:
        fail(f"expected {2 * t} integers, got {len(out)}")
    for case in range(t):
        x, y = inp[1 + 2 * case], inp[2 + 2 * case]
        p, q = out[2 * case], out[2 * case + 1]
        rp, rq = ref[2 * case], ref[2 * case + 1]
        if p < 0 or q < 0:
            fail(f"case {case + 1}: p and q must be non-negative")
        if p & q:
            fail(f"case {case + 1}: {p} & {q} != 0")
        got, want = abs(x - p) + abs(y - q), abs(x - rp) + abs(y - rq)
        if got != want:
            fail(f"case {case + 1}: cost {got}, the optimum is {want}")


if __name__ == "__main__":
    main()
