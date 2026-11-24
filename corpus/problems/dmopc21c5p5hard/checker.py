#!/usr/bin/env python3
"""Checker for DMOPC '21 Contest 5 P5 and its hard version: the output is a permutation of
1..N whose primeness (the sum of the prime gaps between neighbours) equals the answer's."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def primeness(p, sieve):
    return sum(d for d in (abs(a - b) for a, b in zip(p, p[1:])) if sieve[d])


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n = int(open(sys.argv[1]).read().split()[0])
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("non-integer token")
    ans = list(map(int, open(sys.argv[3]).read().split()))
    if sorted(out) != list(range(1, n + 1)):
        fail(f"not a permutation of 1..{n}")
    sieve = [False, False] + [True] * max(0, n - 1)
    for i in range(2, int(n ** 0.5) + 1):
        if sieve[i]:
            sieve[i * i::i] = [False] * len(sieve[i * i::i])
    got, want = primeness(out, sieve), primeness(ans, sieve)
    if got != want:
        fail(f"primeness {got}, want {want}")


main()
