#!/usr/bin/env python3
"""Checker for CF 2084 C: -1 exactly when the reference says -1, else at most n
paired swaps after which a_i = b_{n+1-i} for every i."""
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
        a = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        b = [int(x) for x in inp[pi + 1 + n:pi + 1 + 2 * n]]
        pi += 1 + 2 * n
        none_ref = ref[pr] == "-1"
        pr += 1 if none_ref else 1 + 2 * int(ref[pr])
        if po >= len(out):
            fail(f"case {case}: output ends early")
        if out[po] == "-1":
            if not none_ref:
                fail(f"case {case}: printed -1 but a sequence exists")
            po += 1
            continue
        if none_ref:
            fail(f"case {case}: impossible, expected -1")
        m = int(out[po])
        po += 1
        if not 0 <= m <= n:
            fail(f"case {case}: m = {m} out of range")
        for op in range(1, m + 1):
            i, j = int(out[po]) - 1, int(out[po + 1]) - 1
            po += 2
            if not (0 <= i < n and 0 <= j < n and i != j):
                fail(f"case {case}, op {op}: bad indices {i + 1} {j + 1}")
            a[i], a[j] = a[j], a[i]
            b[i], b[j] = b[j], b[i]
        if a != b[::-1]:
            fail(f"case {case}: a and b are not reverses")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
