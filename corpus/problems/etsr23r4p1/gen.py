#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n, then a permutation p_1..p_n of 1..=n.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e5: 2-cycles and 12 long cycles, 2^12 phase choices
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 100000 if MAX else r.randint(1, 16)
lens, left = [], n
if MAX:
    lens, left = [r.choice([4, 6, 8]) for _ in range(12)], n - 0
    left -= sum(lens)
while left > 0:
    if MAX: l = 2
    else: l = r.choice([1, 2, 2, 3, 4, 4, 6]) if r.random() < 0.9 else r.randint(1, left)
    l = min(l, left)
    lens.append(l)
    left -= l
order = list(range(1, n + 1))
r.shuffle(order)
p = [0] * (n + 1)
at = 0
for l in lens:
    cyc = order[at:at + l]
    at += l
    for i, v in enumerate(cyc):
        p[v] = cyc[(i + 1) % l]
print(n)
print(*p[1:])
