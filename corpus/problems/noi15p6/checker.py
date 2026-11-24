#!/usr/bin/env python3
"""Checker for NOI 2015 P6: the most wishes and the fewest rollers as the reference has them,
and a route of that many trees that is legal: each move goes left, right, up, up-left or up-right
to the nearest tree on that ray not yet wished at."""
import sys
from bisect import bisect_left, bisect_right


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


class Line:
    def __init__(self, coords):
        self.c = sorted(coords)
        self.t = [0] * (len(self.c) + 1)
        for i in range(len(self.c)):
            self.add(i, 1)

    def add(self, i, d):
        i += 1
        while i < len(self.t):
            self.t[i] += d
            i += i & -i

    def prefix(self, i):  # unvisited among the first i
        s = 0
        while i > 0:
            s += self.t[i]
            i -= i & -i
        return s

    def between(self, a, b):  # unvisited with a < coord < b
        lo, hi = bisect_right(self.c, a), bisect_left(self.c, b)
        return self.prefix(hi) - self.prefix(lo) if hi > lo else 0


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, pts = inp[0], list(zip(inp[1::2], inp[2::2]))
    ref = open(sys.argv[3]).read().split()
    out = open(sys.argv[2]).read().split()
    try:
        out = list(map(int, out))
    except ValueError:
        fail("non-integer output")
    m = int(ref[0])
    if not out or out[0] != m:
        fail(f"wishes: got {out[:1]}, want {m}")
    if len(out) != m + 2:
        fail(f"expected {m + 2} numbers, got {len(out)}")
    if out[-1] != int(ref[-1]):
        fail(f"rollers: got {out[-1]}, want {ref[-1]}")
    groups = {}
    for x, y in pts:
        for key, c in ((("y", y), x), (("x", x), y), (("d", x - y), y), (("s", x + y), y)):
            groups.setdefault(key, []).append(c)
    lines = {k: Line(v) for k, v in groups.items()}
    seen = set()
    px, py = 0, 0
    for step, t in enumerate(out[1:-1], 1):
        if not 1 <= t <= n or t in seen:
            fail(f"step {step}: tree {t} is out of range or already wished at")
        qx, qy = pts[t - 1]
        if qy == py:
            key, a, b = ("y", qy), px, qx
        elif qy > py and qx == px:
            key, a, b = ("x", qx), py, qy
        elif qy > py and qx - px == qy - py:
            key, a, b = ("d", qx - qy), py, qy
        elif qy > py and px - qx == qy - py:
            key, a, b = ("s", qx + qy), py, qy
        else:
            fail(f"step {step}: tree {t} is in no allowed direction")
        line = lines[key]
        if line.between(min(a, b), max(a, b)):
            fail(f"step {step}: tree {t} is not the nearest unwished tree that way")
        coord = qx if key[0] == "y" else qy
        for k2, c2 in ((("y", qy), qx), (("x", qx), qy), (("d", qx - qy), qy), (("s", qx + qy), qy)):
            lines[k2].add(bisect_left(lines[k2].c, c2), -1)
        seen.add(t)
        px, py = qx, qy


main()
