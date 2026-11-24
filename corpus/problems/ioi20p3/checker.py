#!/usr/bin/env python3
"""Checker for IOI 2020 Carnival Tickets: the prize matches the answer's, and the allocation
gives each colour exactly one ticket per round 0..k-1 (the rest -1) and pays that prize."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m, k = inp[:3]
    x = [inp[3 + c * m:3 + (c + 1) * m] for c in range(n)]
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if len(out) != 1 + n * m:
        fail(f"expected {1 + n * m} numbers, got {len(out)}")
    try:
        out = list(map(int, out))
    except ValueError:
        fail("output is not integers")
    if out[0] != int(ref[0]):
        fail(f"prize {out[0]}, expected {ref[0]}")
    rounds = [[] for _ in range(k)]
    for c in range(n):
        row = out[1 + c * m:1 + (c + 1) * m]
        if sorted(v for v in row if v != -1) != list(range(k)):
            fail(f"colour {c} does not use each round 0..{k - 1} exactly once")
        for t, v in enumerate(row):
            if v != -1:
                rounds[v].append(x[c][t])
    paid = 0
    for vals in rounds:
        vals.sort()
        paid += sum(vals[n // 2:]) - sum(vals[:n // 2])
    if paid != out[0]:
        fail(f"allocation pays {paid}, prize printed {out[0]}")


main()
