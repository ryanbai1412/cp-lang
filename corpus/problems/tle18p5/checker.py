#!/usr/bin/env python3
"""Checker for TLE '18 P5: the length matches the answer's and the values that follow are a
strictly increasing subsequence of the input of exactly that length."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    a = inp[1:1 + inp[0]]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output is not integers")
    want = int(open(sys.argv[3]).read().split()[0])
    if not out or out[0] != want:
        fail(f"length {out[:1]} but the longest has {want}")
    vals = out[1:]
    if len(vals) != want:
        fail(f"{len(vals)} values for length {want}")
    if any(x >= y for x, y in zip(vals, vals[1:])):
        fail("values are not strictly increasing")
    k = 0
    for x in a:
        if k < len(vals) and x == vals[k]: k += 1
    if k != len(vals):
        fail("values are not a subsequence of the input")


main()
