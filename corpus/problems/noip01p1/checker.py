#!/usr/bin/env python3
"""Checker for NOIP 2001 P1: three roots in increasing order, each within 0.01 of the
reference's (the statement asks for two decimals; a root rounds either way at the edge)."""
import sys


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    out = open(sys.argv[2]).read().split()
    ref = list(map(float, open(sys.argv[3]).read().split()))
    try:
        got = list(map(float, out))
    except ValueError:
        sys.stderr.write("non-numeric output\n")
        sys.exit(1)
    if len(got) != 3:
        sys.stderr.write(f"expected 3 roots, got {len(got)}\n")
        sys.exit(1)
    for i, (g, r) in enumerate(zip(got, ref), 1):
        if abs(g - r) > 0.0101:
            sys.stderr.write(f"root {i}: got {g}, want {r}\n")
            sys.exit(1)


main()
