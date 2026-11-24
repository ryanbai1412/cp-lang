#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; n people, confidences, then `host protocol` per joiner.
import random, sys
MAX = sys.argv[1] == "--max"  # 1e5 people
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 100000 if MAX else r.randint(2, 9)
print(n)
print(*[r.randint(1, 10000 if MAX else 9) for _ in range(n)])
for i in range(1, n):
    print(r.randrange(i), r.randint(0, 2))
