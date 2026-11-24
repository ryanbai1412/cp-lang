#!/usr/bin/env python3
"""Checker for BOI 2019 "Necklace": the length must equal the reference's, and the blocks of s
and t at the printed 0-based starts must be equal after a rotation and maybe a reversal."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    s, t = open(sys.argv[1], encoding="utf-8").read().split()
    out = open(sys.argv[2], encoding="utf-8").read().split()
    ref = open(sys.argv[3], encoding="utf-8").read().split()
    if len(out) != 3:
        fail(f"expected 3 numbers, got {len(out)}")
    try:
        k, i, j = map(int, out)
    except ValueError:
        fail("non-integer output")
    if k != int(ref[0]):
        fail(f"length {k}, reference {ref[0]}")
    if not (0 <= i and i + k <= len(s) and 0 <= j and j + k <= len(t)):
        fail(f"block out of range: {k} {i} {j}")
    a, b = s[i : i + k], t[j : j + k]
    if a not in b + b and a[::-1] not in b + b:
        fail(f"s[{i}..] and t[{j}..] are not rotations")


main()
