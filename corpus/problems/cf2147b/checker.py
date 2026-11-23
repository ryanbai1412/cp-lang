#!/usr/bin/env python3
"""Checker for CF 2147 B: 2n values, each of 1..n twice, the two copies of x a
multiple of x apart."""
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
    t, po = int(inp[0]), 0
    for case, n in enumerate((int(x) for x in inp[1:1 + t]), 1):
        a = [int(x) for x in out[po:po + 2 * n]]
        po += 2 * n
        if len(a) != 2 * n or sorted(a) != sorted(list(range(1, n + 1)) * 2):
            fail(f"case {case}: each of 1..{n} must appear exactly twice")
        first = {}
        for i, x in enumerate(a):
            if x in first and (i - first[x]) % x:
                fail(f"case {case}: the copies of {x} are {i - first[x]} apart")
            first.setdefault(x, i)
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
