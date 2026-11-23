#!/usr/bin/env python3
"""Checker for CF 2084 A: -1 exactly when the reference says -1, else any
permutation with max(p[i-1], p[i]) mod i == i - 1 for 2 <= i <= n."""
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
    inp, out, ref = load()
    t, po, pr = int(inp[0]), 0, 0
    for case in range(1, t + 1):
        n = int(inp[case])
        if po >= len(out):
            fail(f"case {case}: output ends early")
        none_ref = ref[pr] == "-1"
        pr += 1 if none_ref else n
        if out[po] == "-1":
            if not none_ref:
                fail(f"case {case}: printed -1 but a permutation exists")
            po += 1
            continue
        if none_ref:
            fail(f"case {case}: no permutation exists, expected -1")
        p = [int(x) for x in out[po:po + n]]
        po += n
        if sorted(p) != list(range(1, n + 1)):
            fail(f"case {case}: not a permutation of 1..{n}")
        for i in range(2, n + 1):
            if max(p[i - 2], p[i - 1]) % i != i - 1:
                fail(f"case {case}: condition fails at i = {i}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
