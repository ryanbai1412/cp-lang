#!/usr/bin/env python3
"""Checker for Riolku's Mock CCC S3: the swap sequence is not unique, so accept K <= 1e6 allowed
swaps whose application to the input gives the printed permutation, and that permutation equal
to the reference's (the lexicographically least one is unique)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m = inp[0], inp[1]
    p = inp[2:2 + n]
    allowed = {frozenset(inp[2 + n + 2 * i:4 + n + 2 * i]) for i in range(m)}
    ref = open(sys.argv[3]).read().split()
    want = ref[1:1 + n]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    if not out:
        fail("empty output")
    k = out[0]
    if not 0 <= k <= 10**6:
        fail(f"K = {k} is out of 0..=1e6")
    if len(out) != 1 + n + 2 * k:
        fail(f"expected {1 + n + 2 * k} integers, got {len(out)}")
    if [str(x) for x in out[1:1 + n]] != want:
        fail("the permutation is not the least one")
    for i in range(k):
        a, b = out[1 + n + 2 * i], out[2 + n + 2 * i]
        if frozenset((a, b)) not in allowed or a == b:
            fail(f"swap {i + 1} ({a} {b}) is not allowed")
        p[a - 1], p[b - 1] = p[b - 1], p[a - 1]
    if p != out[1:1 + n]:
        fail("the swaps do not produce the printed permutation")


if __name__ == "__main__":
    main()
