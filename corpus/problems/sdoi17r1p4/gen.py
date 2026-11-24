#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SDOI 2017 Freshman Ball. n <= 100, then the n x n tables a and b,
entries in 1..=1e4 (sometimes a narrow range, sometimes b all 1). --max: n = 100."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 100)
n = 100 if MAX else r.randint(1, size)
hi = 10**4 if MAX else r.choice([3, 100, 10**4])
ones = not MAX and r.random() < 0.2
print(n)
for _ in range(n):
    print(*(r.randint(1, hi) for _ in range(n)))
for _ in range(n):
    print(*(1 if ones else r.randint(1, hi) for _ in range(n)))
