#!/usr/bin/env python3
"""Checker for CF 2124 E: -1 exactly when the reference says -1, else the
reference's (minimum) number of operations, each b with 0 <= b_i <= a_i and a
balanced split, together reducing a to zero."""
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
        a = [int(v) for v in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        if po >= len(out):
            fail(f"case {case}: output ends early")
        rs = int(ref[pr])
        pr += 1 if rs == -1 else 1 + rs * n
        s = int(out[po])
        po += 1
        if s != rs:
            fail(f"case {case}: {s} operations, the answer is {rs}")
        for op in range(1, max(s, 0) + 1):
            b = [int(v) for v in out[po:po + n]]
            po += n
            if len(b) != n:
                fail(f"case {case}: output ends early")
            if any(not 0 <= y <= x for x, y in zip(a, b)):
                fail(f"case {case}, op {op}: b exceeds the remaining a")
            total, pre = sum(b), 0
            if not any((pre := pre + y) * 2 == total for y in b[:-1]):
                fail(f"case {case}, op {op}: no balanced split")
            a = [x - y for x, y in zip(a, b)]
        if s != -1 and any(a):
            fail(f"case {case}: elements left nonzero")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
