#!/usr/bin/env python3
# usage: gen.py seed size; budget <= 32000, m <= 60 items, prices 0..=1e4 (multiples of 10, some free)
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1000
n = 60 if big else r.randint(1, min(60, S + 1))
top = 1000 if big else r.choice([10, 100, 1000])
print(32000 if big else r.randint(1, 3200) * 10, n)
mains = [i for i in range(1, n + 1) if i == 1 or r.random() < .4]; left = {i: 2 for i in mains}
for i in range(1, n + 1):
    p = 0
    if i not in left:
        c = [j for j in mains if left[j]]
        if c: p = r.choice(c); left[p] -= 1
        else: left[i] = 0
    price = 0 if r.random() < .15 else r.randint(0, top) * 10
    print(price, r.randint(1, 5), p)
