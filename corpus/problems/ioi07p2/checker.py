#!/usr/bin/env python3
"""Checker for IOI 2007 Flood: the standing walls may be printed in any order, so compare the
count and the set of wall indices with the reference's."""
import sys


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if not out or out[0] != ref[0] or len(out) != len(ref) or sorted(out[1:]) != sorted(ref[1:]):
        sys.stderr.write(f"got {' '.join(out[:6])} ..., want {' '.join(ref[:6])} ...\n")
        sys.exit(1)


main()
