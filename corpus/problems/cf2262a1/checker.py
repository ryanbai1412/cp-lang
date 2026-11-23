#!/usr/bin/env python3
"""Checker for CF 2262 A1: any B, a subset of 0..n-1 listed without repeats, with
mex({floor(y / k) : y in B}) == a_k for every 1 <= k <= n."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = [int(x) for x in open(sys.argv[1], encoding="utf-8").read().split()]
    out = open(sys.argv[2], encoding="utf-8").read().split()
    t, pi, po = inp[0], 1, 0
    for case in range(1, t + 1):
        n = inp[pi]
        a = inp[pi + 1:pi + 1 + n]
        pi += 1 + n
        if po >= len(out):
            fail(f"case {case}: output ends early")
        m = int(out[po])
        b = [int(x) for x in out[po + 1:po + 1 + m]]
        po += 1 + m
        if len(b) != m:
            fail(f"case {case}: output ends early")
        if len(set(b)) != m or any(not 0 <= y < n for y in b):
            fail(f"case {case}: B must be distinct values in 0..{n - 1}")
        for k in range(1, n + 1):
            seen = {y // k for y in b}
            mex = 0
            while mex in seen:
                mex += 1
            if mex != a[k - 1]:
                fail(f"case {case}: f(B, {k}) = {mex}, expected {a[k - 1]}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
