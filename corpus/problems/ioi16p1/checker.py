#!/usr/bin/env python3
"""Checker for IOI 2016 Detecting Molecules: an empty answer exactly when the reference is
empty; otherwise distinct in-range indices whose weights sum into [l, u]."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, l, u = inp[:3]
    w = inp[3:3 + n]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output is not integers")
    ref = list(map(int, open(sys.argv[3]).read().split()))
    if not out or len(out) != 1 + out[0]:
        fail("expected the subset's size, then that many indices")
    picked = out[1:]
    if (ref[0] == 0) != (out[0] == 0):
        fail(f"subset of size {out[0]}, the reference has {ref[0]}")
    if len(set(picked)) != len(picked) or any(not 0 <= i < n for i in picked):
        fail("indices are not distinct and in 0..n")
    total = sum(w[i] for i in picked)
    if picked and not l <= total <= u:
        fail(f"weights sum to {total}, outside [{l}, {u}]")


main()
