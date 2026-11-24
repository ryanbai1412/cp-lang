#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; task, n, then the n final gondola numbers (<= 250000).
# Valid sequences (a rotation, then replacements n+1, n+2, ...); tasks 1 and 3 sometimes broken.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e5, numbers up to 250000
r = random.Random(1 if MAX else int(sys.argv[1]))
task = 3 if MAX else r.randint(1, 3)
n = 10**5 if MAX else r.randint(1, 8)
last = 250000 if MAX else n + r.randint(0, 2 * n)
s = r.randrange(n)
now = [(i + s) % n + 1 for i in range(n)]
for g in range(n + 1, last + 1): now[r.randrange(n)] = g
if not MAX and task != 2 and r.random() < 0.25 and n > 1:  # task 2 inputs are valid
    i, j = r.sample(range(n), 2)
    if r.random() < 0.5: now[i], now[j] = now[j], now[i]
    else: now[i] = now[j]
print(task)
print(n)
print(*now)
