#!/usr/bin/env python3
# checker.py input output [answer]: M and M moves "p m" (1 <= p <= N, m >= 1, at most 10000 moves),
# each legal when made (pile p holds m chips per neighbour), after which every pile holds the same.
import sys
t = open(sys.argv[1]).read().split()
n = int(t[0]); c = [int(v) for v in t[1:1 + n]]
out = open(sys.argv[2]).read().split()
try:
    k = int(out[0]); mv = [(int(out[1 + 2 * i]), int(out[2 + 2 * i])) for i in range(k)]
except (IndexError, ValueError):
    sys.exit(1)
if len(out) != 1 + 2 * k or not 0 <= k <= 10000:
    sys.exit(1)
for p, m in mv:
    if not 1 <= p <= n or m < 1:
        sys.exit(1)
    nb = [q for q in (p - 2, p) if 0 <= q < n]
    if c[p - 1] < m * len(nb):
        sys.exit(1)
    c[p - 1] -= m * len(nb)
    for q in nb:
        c[q] += m
sys.exit(0 if len(set(c)) == 1 else 1)
