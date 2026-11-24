#!/usr/bin/env python3
"""Checker for CEOI 2022 Parking: -1 exactly when the answer is -1; otherwise K equals the
reference's K and the K drives are legal and leave every colour's pair in one space."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m = inp[:2]
    sp = [[x for x in inp[2 + 2 * i:4 + 2 * i] if x] for i in range(m)]
    want = int(open(sys.argv[3]).read().split()[0])
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    if not out:
        fail("empty output")
    k = out[0]
    if k != want:
        fail(f"K = {k}, expected {want}")
    if k == -1:
        if len(out) != 1:
            fail("trailing output after -1")
        sys.exit(0)
    if len(out) != 1 + 2 * k:
        fail("wrong number of drive tokens")
    for q in range(k):
        x, y = out[1 + 2 * q] - 1, out[2 + 2 * q] - 1
        if not (0 <= x < m and 0 <= y < m) or x == y or not sp[x]:
            fail(f"drive {q + 1} is invalid")
        c = sp[x][-1]
        if not (not sp[y] or (len(sp[y]) == 1 and sp[y][0] == c)):
            fail(f"drive {q + 1} parks on a different colour or a full space")
        sp[y].append(sp[x].pop())
    if any(s and (len(s) != 2 or s[0] != s[1]) for s in sp):
        fail("pairs are not together")
    sys.exit(0)


main()
