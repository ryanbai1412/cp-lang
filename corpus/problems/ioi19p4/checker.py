#!/usr/bin/env python3
"""Checker for IOI 2019 P4 (Broken Line): the output is k and k points; with (0, 0) first
they must form axis-parallel segments within |coordinate| <= 2e9 that pass through every
dot, using no more segments than the reference's line (the answer file)."""
import sys
from bisect import bisect_left, bisect_right


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n = inp[0]
    dots = [(inp[1 + 2 * i], inp[2 + 2 * i]) for i in range(n)]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("non-integer token")
    ref_k = int(open(sys.argv[3]).read().split()[0])
    if not out:
        fail("empty output")
    k = out[0]
    if len(out) != 1 + 2 * k:
        fail(f"expected {k} points, got {(len(out) - 1) / 2}")
    if k > ref_k:
        fail(f"{k} segments, the reference uses {ref_k}")
    pts = [(0, 0)] + [(out[1 + 2 * i], out[2 + 2 * i]) for i in range(k)]
    if any(abs(c) > 2 * 10**9 for p in pts for c in p):
        fail("coordinate out of range")
    by_x, by_y = {}, {}
    for x, y in dots:
        by_x.setdefault(x, []).append(y)
        by_y.setdefault(y, []).append(x)
    for v in (*by_x.values(), *by_y.values()):
        v.sort()
    covered = set()
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        if ax == bx:
            col = by_x.get(ax, [])
            lo, hi = sorted((ay, by))
            covered.update((ax, y) for y in col[bisect_left(col, lo):bisect_right(col, hi)])
        elif ay == by:
            row = by_y.get(ay, [])
            lo, hi = sorted((ax, bx))
            covered.update((x, ay) for x in row[bisect_left(row, lo):bisect_right(row, hi)])
        else:
            fail(f"segment ({ax},{ay})-({bx},{by}) is not axis-parallel")
    if len(covered) != n:
        fail(f"{n - len(covered)} dots not on the line")


main()
