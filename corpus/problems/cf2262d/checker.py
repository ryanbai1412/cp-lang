#!/usr/bin/env python3
"""Checker for CF 2262 D: the same -1 verdicts as the original, and every
printed start and operation list (at most 2n^2 operations) turns a into b."""
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
    pi, po, pr = 1, 0, 0
    for case in range(1, t + 1):
        n = inp[pi]
        a = inp[pi + 1:pi + 1 + n]
        b = inp[pi + 1 + n:pi + 1 + 2 * n]
        pi += 1 + 2 * n
        if po >= len(out):
            fail(f"case {case}: output ends early")
        impossible = ref[pr] == -1
        pr += 1 if impossible else 2 + ref[pr]
        if (out[po] == -1) != impossible:
            fail(f"case {case}: -1 verdict differs from the original")
        if impossible:
            po += 1
            continue
        o, p = out[po], out[po + 1]
        ops = out[po + 2:po + 2 + o]
        po += 2 + o
        if not 0 <= o <= 2 * n * n or len(ops) != o:
            fail(f"case {case}: {o} operations, at most {2 * n * n} allowed")
        if not 1 <= p <= n:
            fail(f"case {case}: start {p} out of 1..{n}")
        a = [0] + a
        for c in ops:
            if c == 1:
                if a[p] >= n:
                    fail(f"case {case}: operation 1 at barn {p} with a_p = n")
                a[p] += 1
            elif c == 2:
                p = a[p]
            else:
                fail(f"case {case}: operation type {c}")
        if a[1:] != b:
            fail(f"case {case}: a does not end equal to b")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
