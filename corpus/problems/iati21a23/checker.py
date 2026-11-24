#!/usr/bin/env python3
# checker.py input output [answer]: a count K and K shots in 1..=n after which the target can be nowhere
# (accepted submissions differ in K). Before shot i the target is anywhere still possible; a shot at x
# rules out x, then for i mod (s + c) < s every other cell steps away from x, else towards it (walls clamp),
# as in the accepted 5072008.cpp's own check.
import sys
n, s, c = map(int, open(sys.argv[1]).read().split()[:3])
out = open(sys.argv[2]).read().split()
try:
    k = int(out[0]); shots = [int(x) for x in out[1:]]
except (IndexError, ValueError):
    sys.exit(1)
if len(shots) != k or any(not 1 <= x <= n for x in shots):
    sys.exit(1)
live = [True] * n
for i, x in enumerate(shots):
    x -= 1
    away = i % (s + c) < s
    nxt = [False] * n
    for j in range(n):
        if live[j] and j != x:
            left = j < x
            nxt[max(0, j - 1) if left == away else min(n - 1, j + 1)] = True
    live = nxt
sys.exit(1 if any(live) else 0)
