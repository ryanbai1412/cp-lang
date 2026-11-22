#!/usr/bin/env python3
import sys, random
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(2, 10)
    all_pairs = [(i, x) for i in range(1, n + 1) for x in range(1, n + 1)]
    m = rng.randint(0, min(len(all_pairs), 12))
    chosen = rng.sample(all_pairs, m)
    print(n, m)
    for i, x in chosen:
        print(i, x)
