#!/usr/bin/env python3
"""Checker for CF 2157 C: any 0 <= a_i <= 1e9 meeting every min (c = 1) and
MEX (c = 2) constraint with the shared k."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def mex(xs):
    seen, m = set(xs), 0
    while m in seen:
        m += 1
    return m


def main():
    inp, out, _ = load()
    t, pi, po = int(inp[0]), 1, 0
    for case in range(1, t + 1):
        n, k, q = (int(v) for v in inp[pi:pi + 3])
        pi += 3
        a = [int(v) for v in out[po:po + n]]
        po += n
        if len(a) != n or any(not 0 <= v <= 10**9 for v in a):
            fail(f"case {case}: expected {n} values in [0, 1e9]")
        for j in range(q):
            c, l, r = (int(v) for v in inp[pi:pi + 3])
            pi += 3
            seg = a[l - 1:r]
            got = min(seg) if c == 1 else mex(seg)
            if got != k:
                fail(f"case {case}: {'min' if c == 1 else 'MEX'} of [{l}, {r}] is {got}, not {k}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
