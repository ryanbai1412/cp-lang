#!/usr/bin/env python3
"""Checker for CEOI 2022 Drawing: the output is a permutation (label of the node drawn at each
input point) and no two tree edges, drawn as segments, cross. Points are in general position, so
two edges sharing a node meet only there and any other contact is a proper crossing; a
Shamos-Hoey sweep over the points in (x, y) order finds one if it exists."""
import sys
from functools import cmp_to_key


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n = inp[0]
    ed = inp[1:2 * n - 1]
    pts = inp[2 * n - 1:]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    if len(out) != n or sorted(out) != list(range(1, n + 1)):
        fail("output is not a permutation of 1..N")
    at = [0] * (n + 1)
    for i, v in enumerate(out):
        at[v] = i
    X = pts[0::2]
    Y = pts[1::2]
    order = sorted(range(n), key=lambda i: (X[i], Y[i]))
    rank = [0] * n
    for k, i in enumerate(order):
        rank[i] = k
    segs = []
    starts = [[] for _ in range(n)]
    ends = [[] for _ in range(n)]
    for k in range(n - 1):
        a, b = at[ed[2 * k]], at[ed[2 * k + 1]]
        if rank[a] > rank[b]:
            a, b = b, a
        starts[a].append(len(segs))
        ends[b].append(len(segs))
        segs.append((a, b))

    def turn(o, a, b):
        return (X[a] - X[o]) * (Y[b] - Y[o]) - (Y[a] - Y[o]) * (X[b] - X[o])

    def side(s, p):
        a, b = segs[s]
        return (X[b] - X[a]) * (Y[p] - Y[a]) - (Y[b] - Y[a]) * (X[p] - X[a])

    def cross(s, t):
        a, b = segs[s]
        c, d = segs[t]
        if a in (c, d) or b in (c, d):
            return False
        return (side(s, c) > 0) != (side(s, d) > 0) and (side(t, a) > 0) != (side(t, b) > 0)

    st = []
    for p in order:
        lo, hi = 0, len(st)
        while lo < hi:
            m = (lo + hi) // 2
            if side(st[m], p) > 0:
                lo = m + 1
            else:
                hi = m
        i = j = lo
        while j < len(st) and segs[st[j]][1] == p:
            j += 1
        if j - i != len(ends[p]):
            fail("edges cross")
        del st[i:j]
        new = sorted(starts[p], key=cmp_to_key(lambda s, t: -turn(p, segs[s][1], segs[t][1])))
        if new:
            if any(turn(p, segs[s][1], segs[t][1]) == 0 for s, t in zip(new, new[1:])):
                fail("edges overlap")
            st[i:i] = new
            if i > 0 and cross(st[i - 1], st[i]):
                fail("edges cross")
            k = i + len(new)
            if k < len(st) and cross(st[k - 1], st[k]):
                fail("edges cross")
        elif 0 < i < len(st) and cross(st[i - 1], st[i]):
            fail("edges cross")
    sys.exit(0)


main()
