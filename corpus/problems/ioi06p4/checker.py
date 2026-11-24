#!/usr/bin/env python3
"""Checker for IOI 2006 Valley of Mexico: -1 exactly when the answer is -1; otherwise c
distinct cities, consecutive ones agreeing, and no two route segments crossing. Cities sit
in convex position, so a route never crosses itself iff every city it visits extends the arc
of cities visited so far at one of its ends."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    c, n = inp[0], inp[1]
    deals = {(min(u, v), max(u, v)) for u, v in zip(inp[2:2 + 2 * n:2], inp[3:3 + 2 * n:2])}
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if ref[:1] == ["-1"] or out[:1] == ["-1"]:
        if out != ref[:1]:
            fail(f"got {' '.join(out[:1])}, want {ref[0]}")
        return
    try:
        route = list(map(int, out))
    except ValueError:
        fail("route must be integers")
    if len(route) != c or sorted(route) != list(range(1, c + 1)):
        fail(f"want each of the {c} cities once")
    lo = hi = route[0] - 1
    for a, b in zip(route, route[1:]):
        if (min(a, b), max(a, b)) not in deals:
            fail(f"no agreement between {a} and {b}")
        if b - 1 == (lo - 1) % c:
            lo = b - 1
        elif b - 1 == (hi + 1) % c:
            hi = b - 1
        else:
            fail(f"segment {a}-{b} crosses the route")


if __name__ == "__main__":
    main()
