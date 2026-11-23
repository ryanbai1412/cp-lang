#!/usr/bin/env python3
"""Checker for CF 2261 D1: the same number of added detectors as the original,
each a segment inside 1..n, and with the given ones every spot gets a distinct
set of covering detectors (compared by a random 64-bit hash per detector)."""
import random
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = [int(x) for x in open(sys.argv[1], encoding="utf-8").read().split()]
    out = [int(x) for x in open(sys.argv[2], encoding="utf-8").read().split()]
    ref = [int(x) for x in open(sys.argv[3], encoding="utf-8").read().split()]
    rng = random.Random(2261)
    mask = (1 << 64) - 1
    t = inp[0]
    pi, po, pr = 1, 0, 0
    for case in range(1, t + 1):
        n, m = inp[pi], inp[pi + 1]
        segs = [(inp[pi + 2 + 2 * k], inp[pi + 3 + 2 * k]) for k in range(m)]
        pi += 2 + 2 * m
        if po >= len(out):
            fail(f"case {case}: output ends early")
        k, want = out[po], ref[pr]
        pr += 1 + 2 * want
        if k != want:
            fail(f"case {case}: {k} detectors added, the minimum is {want}")
        added = [(out[po + 1 + 2 * j], out[po + 2 + 2 * j]) for j in range(k)]
        po += 1 + 2 * k
        if len(added) != k or any(len(s) != 2 for s in added):
            fail(f"case {case}: output ends early")
        diff = [0] * (n + 2)
        for u, v in segs + added:
            if not 1 <= u <= v <= n:
                fail(f"case {case}: detector [{u}, {v}] outside 1..{n}")
            h = rng.getrandbits(64)
            diff[u] = (diff[u] + h) & mask
            diff[v + 1] = (diff[v + 1] - h) & mask
        sig, acc = set(), 0
        for i in range(1, n + 1):
            acc = (acc + diff[i]) & mask
            if acc in sig:
                fail(f"case {case}: spot {i} has the same reports as an earlier spot")
            sig.add(acc)
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
