#!/usr/bin/env python3
"""Checker for CF 2245 D2: the same YES/NO verdicts as the original, and every
printed array has |a_i| <= 1e9 and meets each restriction (o = 1: a_i + a_j >= 0,
o = 2: a_i + a_j < 0)."""
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
    ref = open(sys.argv[3], encoding="utf-8").read().split()
    t = inp[0]
    pi, po, pr = 1, 0, 0
    for case in range(1, t + 1):
        n, m = inp[pi], inp[pi + 1]
        rules = [inp[pi + 2 + 3 * k:pi + 5 + 3 * k] for k in range(m)]
        pi += 2 + 3 * m
        if po >= len(out):
            fail(f"case {case}: output ends early")
        got, want = out[po].upper(), ref[pr].upper()
        po += 1
        pr += 1 + (n if want == "YES" else 0)
        if got not in ("YES", "NO"):
            fail(f"case {case}: expected YES or NO, got {out[po - 1]!r}")
        if got != want:
            fail(f"case {case}: verdict {got}, original says {want}")
        if got == "NO":
            continue
        a = [0] + [int(x) for x in out[po:po + n]]
        po += n
        if len(a) != n + 1 or any(abs(x) > 10**9 for x in a):
            fail(f"case {case}: need {n} values with |a_i| <= 1e9")
        for o, i, j in rules:
            s = a[i] + a[j]
            if (o == 1 and s < 0) or (o == 2 and s >= 0):
                fail(f"case {case}: restriction ({o}, {i}, {j}) broken, a_i + a_j = {s}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
