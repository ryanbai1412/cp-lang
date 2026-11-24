#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n, then a permutation of 1..n.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e6
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else r.randint(1, 9)
p = list(range(1, n + 1)); r.shuffle(p)
print(n)
print("\n".join(map(str, p)))
