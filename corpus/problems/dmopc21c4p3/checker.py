#!/usr/bin/env python3
"""Checker for DMOPC '21 Contest 4 P3: ten distinct non-negative integers summing to at most
1e18, every subset sum a palindrome, and the total equal to the answer's (the minimum)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    toks = open(sys.argv[2]).read().split()
    if len(toks) != 10 or not all(t.isdigit() for t in toks):
        fail("expected 10 non-negative integers")
    xs = list(map(int, toks))
    want = sum(map(int, open(sys.argv[3]).read().split()))
    if len(set(xs)) != 10:
        fail("the integers are not distinct")
    if sum(xs) > 10**18:
        fail("the sum exceeds 1e18")
    sums = [0]
    for x in xs:
        sums += [s + x for s in sums]
    bad = next((s for s in sums if str(s) != str(s)[::-1]), None)
    if bad is not None:
        fail(f"subset sum {bad} is not a palindrome")
    if sum(xs) != want:
        fail(f"sum {sum(xs)}, want {want}")


main()
