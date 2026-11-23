#!/usr/bin/env python3
"""Checker for CF 2101 A: any arrangement of 0..n^2-1 whose MEX sum over all
subgrids equals the reference's. The sum is, over k >= 1, the number of
subgrids holding all of 0..k-1, counted from the bounding box of their cells."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def mex_sum(n, g):
    where = [None] * (n * n)
    for i in range(n):
        for j in range(n):
            where[g[i][j]] = (i + 1, j + 1)
    r1 = c1 = n + 1
    r2 = c2 = 0
    total = 0
    for k in range(n * n):
        r, c = where[k]
        r1, r2, c1, c2 = min(r1, r), max(r2, r), min(c1, c), max(c2, c)
        total += r1 * (n - r2 + 1) * c1 * (n - c2 + 1)
    return total


def main():
    inp, out, ref = load()
    t, po = int(inp[0]), 0
    for case, n in enumerate((int(x) for x in inp[1:1 + t]), 1):
        cells = [int(x) for x in out[po:po + n * n]]
        want = [int(x) for x in ref[po:po + n * n]]
        po += n * n
        if len(cells) != n * n or sorted(cells) != list(range(n * n)):
            fail(f"case {case}: not an arrangement of 0..{n * n - 1}")
        got = mex_sum(n, [cells[i * n:(i + 1) * n] for i in range(n)])
        best = mex_sum(n, [want[i * n:(i + 1) * n] for i in range(n)])
        if got != best:
            fail(f"case {case}: MEX sum {got}, the optimum is {best}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
