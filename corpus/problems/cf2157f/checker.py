#!/usr/bin/env python3
"""Checker for CF 2157 F: at most 1e6 missions with 1 <= y, l <= 1e6, total cost
at most 1e6, after which every starting skill in 1..n has reached n. The
starting skills are tracked as counts per current skill."""
import sys
from collections import Counter


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n = int(open(sys.argv[1]).read().split()[0])
    out = open(sys.argv[2]).read().split()
    k = int(out[0])
    if not 0 <= k <= 10**6 or len(out) != 1 + 2 * k:
        fail(f"expected k in [0, 1e6] and 2k values, got k = {k}, {len(out) - 1} values")
    skill = Counter(range(1, n + 1))
    cost, last = 0, None
    for i in range(k):
        y, l = int(out[1 + 2 * i]), int(out[2 + 2 * i])
        if not (1 <= y <= 10**6 and 1 <= l <= 10**6):
            fail(f"mission {i + 1}: ({y}, {l}) out of range")
        cost += l + (1000 if last is not None and y > last else 0)
        last = y
        if y in skill and y < n:
            skill[y + l] += skill.pop(y)
    if cost > 10**6:
        fail(f"cost {cost} exceeds 1e6")
    low = min(skill)
    if low < n:
        fail(f"a start ends at skill {low} < {n}")


if __name__ == "__main__":
    main()
