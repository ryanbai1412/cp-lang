#!/usr/bin/env python3
"""Checker for CCO 2018 P6: at most 3e5 flops `l r` (1 <= l <= r <= N), each swapping the
minimum and maximum of a_l..a_r, that turn a into b."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n = inp[0]
    a, b = inp[1:1 + n], inp[1 + n:1 + 2 * n]
    out = open(sys.argv[2]).read().split()
    if not out:
        fail("empty output")
    k = int(out[0])
    if not 0 <= k <= 300_000 or len(out) != 1 + 2 * k:
        fail(f"{k} flops in {len(out)} tokens")
    for t in range(k):
        l, r = int(out[1 + 2 * t]), int(out[2 + 2 * t])
        if not 1 <= l <= r <= n:
            fail(f"flop {t + 1}: {l} {r} outside 1..={n}")
        seg = a[l - 1:r]
        i, j = l - 1 + seg.index(min(seg)), l - 1 + seg.index(max(seg))
        a[i], a[j] = a[j], a[i]
    if a != b:
        fail("the flops do not turn a into b")


main()
