#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; an n x m chocolate, 1..=1000 raisins per piece.
import random, sys
MAX = sys.argv[1] == "--max"  # 50 x 50
r = random.Random(1 if MAX else int(sys.argv[1]))
n, m = (50, 50) if MAX else (r.randint(1, 6), r.randint(1, 6))
print(n, m)
for _ in range(n):
    print(*[r.randint(1, 1000) for _ in range(m)])
