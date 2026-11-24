#!/usr/bin/env python3
"""Hidden test for CCO 2009 P4: `gen.py SEED SIZE | --max` prints n, then the n cap sizes, then
the n bottle sizes (a shuffle of the same distinct sizes). --max is n = 1e4."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10_000 if MAX else r.randint(1, 2 + 3 * S)
sizes = r.sample(range(1, 10 * n + 1), n)
bottles = sizes[:]; r.shuffle(bottles)
print(n)
print(" ".join(map(str, sizes)))
print(" ".join(map(str, bottles)))
