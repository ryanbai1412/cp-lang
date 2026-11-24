#!/usr/bin/env python3
"""Checker for DMOPC '21 C5 P1: -1 exactly when the answer is -1; otherwise a permutation
of 1..=n in which no two neighbours sum to a prime."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    n = int(open(sys.argv[1]).read().split()[0])
    out = list(map(int, open(sys.argv[2]).read().split()))
    want = open(sys.argv[3]).read().split()[0]
    if want == "-1" or out[:1] == [-1]:
        if out != [-1] or want != "-1":
            fail(f"output starts {out[:1]}, answer says {want}")
        return
    if sorted(out) != list(range(1, n + 1)):
        fail("not a permutation of 1..=n")
    top = 2 * n
    composite = bytearray(top + 1)
    composite[0] = composite[1] = 1
    for p in range(2, int(top ** 0.5) + 1):
        if not composite[p]:
            composite[p * p::p] = b"\x01" * len(range(p * p, top + 1, p))
    for a, b in zip(out, out[1:]):
        if not composite[a + b]:
            fail(f"{a} + {b} is prime")


main()
