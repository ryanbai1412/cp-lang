#!/usr/bin/env python3
"""Checker for CF 2089 A: a permutation whose running ceil-means include at
least floor(n / 3) - 1 primes."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def main():
    inp, out, _ = load()
    t, po = int(inp[0]), 0
    ns = [int(x) for x in inp[1:1 + t]]
    top = max(ns)
    sieve = bytearray([1]) * (top + 2)
    sieve[0:2] = b"\x00\x00"
    for i in range(2, int(top ** 0.5) + 2):
        if i < len(sieve) and sieve[i]:
            sieve[i * i::i] = bytearray(len(sieve[i * i::i]))
    for case, n in enumerate(ns, 1):
        p = [int(x) for x in out[po:po + n]]
        po += n
        if len(p) != n or sorted(p) != list(range(1, n + 1)):
            fail(f"case {case}: not a permutation of 1..{n}")
        s = primes = 0
        for i, v in enumerate(p, 1):
            s += v
            c = -(-s // i)
            primes += c < len(sieve) and sieve[c]
        if primes < n // 3 - 1:
            fail(f"case {case}: {primes} primes, need {n // 3 - 1}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
