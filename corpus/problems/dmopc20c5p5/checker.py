#!/usr/bin/env python3
"""Checker for DMOPC '20 Contest 5 P5: the output is an N x M grid of Y and B with no ugly
sub-rectangle (equal first and last rows and equal first and last columns, both sides >= 2).
Zero is always reachable (shifted Thue-Morse, official editorial), so any ugly one fails.

For each top row r1 the columns are split into classes with equal content on rows r1..r2,
as bitmasks; a class with two members inside one run of columns where rows r1 and r2 agree
is an ugly rectangle. Singleton classes are dropped, so the scan is near linear."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n, m = map(int, open(sys.argv[1]).read().split()[:2])
    g = open(sys.argv[2]).read().split()
    if len(g) != n or any(len(r) != m or set(r) - set("YB") for r in g):
        fail(f"expected {n} rows of {m} Y/B characters")
    if n > m:
        g = ["".join(col) for col in zip(*g)]
        n, m = m, n
    full = (1 << m) - 1
    rows = [int(r[::-1].replace("Y", "1").replace("B", "0"), 2) for r in g]
    for r1 in range(n):
        top = rows[r1]
        classes = [k for k in (top, full & ~top) if k.bit_count() >= 2]
        for r2 in range(r1 + 1, n):
            if not classes:
                break
            row = rows[r2]
            classes = [k for c in classes for k in (c & row, c & ~row) if k.bit_count() >= 2]
            zero = (top ^ row) & full
            for c in classes:
                k = c & ~zero
                while k & (k - 1):
                    a = k & -k
                    k ^= a
                    b = k & -k
                    if zero & (b - a) == 0:
                        ca, cb = a.bit_length() - 1, b.bit_length() - 1
                        fail(f"ugly rectangle rows {r1 + 1}..{r2 + 1} cols {ca + 1}..{cb + 1} (maybe transposed)")


main()
