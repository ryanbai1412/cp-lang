#!/usr/bin/env python3
"""Checker for NOIP 1999 P4: the optimal value set is not unique (the accepted C++ answers
disagree on `2 2`), so accept K increasing face values starting at 1 whose N-stamp sums cover
1..=MAX, with `MAX=` the reference's maximum."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n, k = map(int, open(sys.argv[1]).read().split())
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    want = int(ref[-1].removeprefix("MAX="))
    if len(out) != k + 1 or not out[-1].startswith("MAX="):
        fail(f"expected {k} values and MAX=, got {' '.join(out[:k + 2])}")
    try:
        vals = [int(x) for x in out[:k]]
        got = int(out[-1].removeprefix("MAX="))
    except ValueError:
        fail("values must be integers")
    if got != want:
        fail(f"MAX={got}, want MAX={want}")
    if any(b <= a for a, b in zip(vals, vals[1:])) or vals[0] < 1:
        fail("face values must be positive and increasing")
    inf = n + 1
    need = [0] + [inf] * (want + 1)
    for v in range(1, want + 2):
        need[v] = min((need[v - x] + 1 for x in vals if x <= v), default=inf)
    if any(need[v] > n for v in range(1, want + 1)):
        fail(f"{' '.join(out[:k])} does not cover 1..={want} with {n} stamps")


if __name__ == "__main__":
    main()
