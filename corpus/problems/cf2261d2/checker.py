#!/usr/bin/env python3
"""Checker for CF 2261 D2: the reference's (minimum) count of new detectors, each
a segment in 1..n, after which every spot has a distinct report vector. A
spot's vector is hashed as the sum of random 64-bit weights of the detectors
covering it, so two spots collide only if their vectors match (or with
negligible probability)."""
import random
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
    inp, out, ref = load()
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    rng = random.Random(2261)
    for case in range(1, t + 1):
        n, m = int(inp[pi]), int(inp[pi + 1])
        segs = [(int(inp[pi + 2 + 2 * j]), int(inp[pi + 3 + 2 * j])) for j in range(m)]
        pi += 2 + 2 * m
        rk = int(ref[pr])
        pr += 1 + 2 * rk
        k = int(out[po])
        if k != rk:
            fail(f"case {case}: {k} new detectors, the minimum is {rk}")
        for j in range(k):
            u, v = int(out[po + 1 + 2 * j]), int(out[po + 2 + 2 * j])
            if not 1 <= u <= v <= n:
                fail(f"case {case}: detector {u} {v} out of range")
            segs.append((u, v))
        po += 1 + 2 * k
        diff = [0] * (n + 2)
        for l, r in segs:
            w = rng.getrandbits(64)
            diff[l] += w
            diff[r + 1] -= w
        seen, h = {}, 0
        for spot in range(1, n + 1):
            h += diff[spot]
            if h in seen:
                fail(f"case {case}: spots {seen[h]} and {spot} give the same reports")
            seen[h] = spot
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
