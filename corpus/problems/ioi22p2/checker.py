#!/usr/bin/env python3
"""Checker for IOI 2022 Prisoner Challenge: at most 21 states (x <= 20), every action in
range, and for each tested pair of distinct counts the prisoners name the smaller bag. All
pairs when n <= 200, else every pair at distance 1 or 2 and 100000 seeded random ones."""
import random
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n = int(open(sys.argv[1]).read().split()[0])
    try:
        out = [int(t) for t in open(sys.argv[2]).read().split()]
    except ValueError:
        fail("output is not integers")
    if not out:
        fail("empty output")
    m = out[0]
    if not 1 <= m <= 21 or len(out) != 1 + m * (n + 1):
        fail(f"{m} states, or the wrong number of entries")
    rows = [out[1 + i * (n + 1):1 + (i + 1) * (n + 1)] for i in range(m)]
    for i, row in enumerate(rows):
        if row[0] not in (0, 1) or any(not -2 <= x < m for x in row[1:]):
            fail(f"state {i} has an entry out of range")
    if n <= 200:
        pairs = [(a, b) for a in range(1, n + 1) for b in range(1, n + 1) if a != b]
    else:
        r = random.Random(n)
        pairs = [(v, v + d) for d in (1, 2) for v in range(1, n + 1 - d)]
        pairs += [(b, a) for a, b in pairs]
        pairs += [tuple(r.sample(range(1, n + 1), 2)) for _ in range(100_000)]
    for a, b in pairs:
        state = 0
        for _ in range(m * 50):
            row = rows[state]
            act = row[a if row[0] == 0 else b]
            if act < 0:
                if (act == -1) != (a < b):
                    fail(f"A = {a}, B = {b}: names the larger bag")
                break
            state = act
        else:
            fail(f"A = {a}, B = {b}: never names a bag")


main()
