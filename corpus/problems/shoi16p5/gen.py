#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SHOI 2016 Random Sequence. n, Q <= 1e5, a_i <= 1e4, then Q updates
`t v` with v <= 1e4 (zeros included). --max: n = Q = 1e5."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 10**5)
n, q = (size, size) if MAX else (r.randint(1, size), r.randint(1, size))
hi = r.choice([2, 10**4])
val = lambda: 0 if r.random() < 0.1 else r.randint(1, hi)
print(n, q)
print(*(val() for _ in range(n)))
print("\n".join(f"{r.randint(1, n)} {val()}" for _ in range(q)))
