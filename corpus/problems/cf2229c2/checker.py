#!/usr/bin/env python3
"""Checker for CF 2229 C2: at most n prefix flips, each at a positive a_i, whose
final sum equals the reference's (maximal) final sum."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def final(a, ops, case, who):
    """The final sum: a_j ends negated once per flip at an index >= j. A Fenwick tree over
    the flips so far gives the sign at each flip, O(k log n)."""
    n = len(a)
    tree = [0] * (n + 1)
    for i in ops:
        if not 1 <= i <= n:
            fail(f"case {case}: {who} flips at {i}, out of range")
        later, p = 0, n
        while p > 0:
            later += tree[p]
            p -= p & -p
        p = i - 1
        while p > 0:
            later -= tree[p]
            p -= p & -p
        if (a[i - 1] if later % 2 == 0 else -a[i - 1]) <= 0:
            fail(f"case {case}: {who} flips at {i}, where a_i is not positive")
        p = i
        while p <= n:
            tree[p] += 1
            p += p & -p
    flips = [0] * (n + 2)
    for i in ops:
        flips[i] += 1
    total, above = 0, 0
    for j in range(n, 0, -1):
        above += flips[j]
        total += a[j - 1] if above % 2 == 0 else -a[j - 1]
    return total


def main():
    inp, out, ref = load()
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        a = [int(x) for x in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        k, rk = int(out[po]), int(ref[pr])
        ops = [int(x) for x in out[po + 1:po + 1 + k]]
        rops = [int(x) for x in ref[pr + 1:pr + 1 + rk]]
        po, pr = po + 1 + k, pr + 1 + rk
        if not 0 <= k <= n or len(ops) != k:
            fail(f"case {case}: k = {k} out of range")
        got, best = final(a, ops, case, "output"), final(a, rops, case, "reference")
        if got != best:
            fail(f"case {case}: final sum {got}, the maximum is {best}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
