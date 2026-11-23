#!/usr/bin/env python3
"""Checker for CF 2138 F: at most m operations, each on an existing segment with
1 <= u, v <= i + 1 and u != v, the new point inside [-2e4, 2e4]^2, every side of
the new triangle in [0.5, 1] (1e-8 slack), and some point within 1e-4 of (p, q)."""
import math
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
    lo, hi = 0.5 - 1e-8, 1 + 1e-8
    t, pi, po = inp[0], 1, 0
    for case in range(1, t + 1):
        p, q, m = inp[pi:pi + 3]
        pi += 3
        if po >= len(out):
            fail(f"case {case}: output ends early")
        n = int(out[po])
        po += 1
        if not 0 <= n <= m:
            fail(f"case {case}: {n} operations, the limit is {m}")
        pts = [(0.0, 0.0), (1.0, 0.0)]
        segs = {(0, 1)}
        for i in range(1, n + 1):
            if po + 4 > len(out):
                fail(f"case {case}: output ends early")
            u, v = int(out[po]) - 1, int(out[po + 1]) - 1
            x, y = float(out[po + 2]), float(out[po + 3])
            po += 4
            if not (0 <= u <= i and 0 <= v <= i and u != v):
                fail(f"case {case}, op {i}: bad identifiers {u + 1} {v + 1}")
            if (min(u, v), max(u, v)) not in segs:
                fail(f"case {case}, op {i}: {u + 1} and {v + 1} are not joined by a segment")
            if not (abs(x) <= 2e4 and abs(y) <= 2e4):
                fail(f"case {case}, op {i}: point ({x}, {y}) out of range")
            c = len(pts)
            pts.append((x, y))
            for a, b in ((u, v), (u, c), (v, c)):
                d = math.dist(pts[a], pts[b])
                if not lo <= d <= hi:
                    fail(f"case {case}, op {i}: side {a + 1}-{b + 1} has length {d}")
            segs.add((min(u, c), c))
            segs.add((min(v, c), c))
        if not any(math.dist(pt, (p, q)) <= 1e-4 for pt in pts):
            fail(f"case {case}: no point within 1e-4 of ({p}, {q})")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
