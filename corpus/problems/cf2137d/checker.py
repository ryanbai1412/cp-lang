#!/usr/bin/env python3
"""Checker for CF 2137 D: -1 exactly when the reference says -1, else any a with
1 <= a_i <= n and f(a_i) == b_i, where f counts occurrences."""
from collections import Counter
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
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        b = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        none_ref = ref[pr] == "-1"
        pr += 1 if none_ref else n
        if po >= len(out):
            fail(f"case {case}: output ends early")
        if out[po] == "-1":
            if not none_ref:
                fail(f"case {case}: printed -1 but an array exists")
            po += 1
            continue
        if none_ref:
            fail(f"case {case}: no array exists, expected -1")
        a = [int(x) for x in out[po:po + n]]
        po += n
        if len(a) != n or any(not 1 <= x <= n for x in a):
            fail(f"case {case}: expected {n} values in 1..{n}")
        f = Counter(a)
        for i in range(n):
            if f[a[i]] != b[i]:
                fail(f"case {case}: f(a_{i + 1}) = {f[a[i]]}, b_{i + 1} = {b[i]}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
