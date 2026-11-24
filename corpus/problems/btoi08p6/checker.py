#!/usr/bin/env python3
"""Checker for BOI 2008 P6 (Gloves): L + R equals the answer's, L and R fit the drawers, and
no colour set S has L <= its left gloves and R <= the right gloves of the other colours."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    t = list(map(int, open(sys.argv[1]).read().split()))
    n = t[0]
    a, b = t[1:1 + n], t[1 + n:1 + 2 * n]
    out = open(sys.argv[2]).read().split()
    ref = list(map(int, open(sys.argv[3]).read().split()))
    if len(out) != 2 or not all(w.isdigit() for w in out):
        fail(f"expected 2 integers, got {' '.join(out)[:60]!r}")
    lft, rgt = map(int, out)
    if lft + rgt != sum(ref):
        fail(f"{lft} + {rgt} gloves, want {sum(ref)}")
    if lft > sum(a) or rgt > sum(b):
        fail(f"{lft} left or {rgt} right is more than the drawer holds")
    sa, sb = [0] * (1 << n), [0] * (1 << n)
    for s in range(1, 1 << n):
        low = (s & -s).bit_length() - 1
        sa[s], sb[s] = sa[s & (s - 1)] + a[low], sb[s & (s - 1)] + b[low]
    full = (1 << n) - 1
    for s in range(1 << n):
        if lft <= sa[s] and rgt <= sb[full ^ s]:
            fail(f"colours {s:b} may take all {lft} left and the others all {rgt} right")


main()
