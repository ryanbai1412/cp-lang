#!/usr/bin/env python3
"""Checker for DMOPC '20 C7 P1: the count matches the answer's, and the n lines pair up
every weight 1..=2n exactly once with that many unequal pairs."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, mass = inp[0], inp[1:]
    out = list(map(int, open(sys.argv[2]).read().split()))
    want = int(open(sys.argv[3]).read().split()[0])
    if len(out) != 1 + 2 * n:
        fail(f"expected {1 + 2 * n} tokens, got {len(out)}")
    if out[0] != want:
        fail(f"count {out[0]}, want {want}")
    if sorted(out[1:]) != list(range(1, 2 * n + 1)):
        fail("the pairs do not use every weight exactly once")
    tipped = sum(mass[out[1 + 2 * i] - 1] != mass[out[2 + 2 * i] - 1] for i in range(n))
    if tipped != out[0]:
        fail(f"the pairs tip {tipped} scales, the count says {out[0]}")


main()
