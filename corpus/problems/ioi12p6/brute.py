#!/usr/bin/env python3
# Brute force: seat the late knight at every position and replay the rounds. O(n^2 c).
import sys
v = sys.stdin.read().split(); n, c, r = int(v[0]), int(v[1]), int(v[2])
ranks = list(map(int, v[3:3 + n - 1])); qs = list(map(int, v[3 + n - 1:]))
best = (-1, 0)
for p in range(n):
    line = ranks[:p] + [r] + ranks[p:]; wins = 0
    for i in range(c):
        s, e = qs[2 * i], qs[2 * i + 1]
        seg = line[s:e + 1]; m = max(seg)
        if m == r: wins += 1
        line = line[:s] + [m] + line[e + 1:]
    if wins > best[0]: best = (wins, p)
print(best[1])
