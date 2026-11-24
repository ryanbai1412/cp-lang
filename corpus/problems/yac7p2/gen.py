#!/usr/bin/env python3
"""gen.py seed size | --max: YAC7 P2. T test cases, each an n x m grid, T, n, m <= 100.
--max: T = 100 grids of 100 x 100 and the other parity mixes (odd x odd, odd x even, 1 x m)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 100 if MAX else max(1, min(100, int(sys.argv[2]) // 5))
t = 100 if MAX else r.randint(1, 10)
cases = []
for i in range(t):
    if MAX:
        n, m = [(100, 100), (99, 99), (99, 100), (100, 99), (1, 100)][i % 5] if i < 95 else (100, 100)
    else:
        n, m = r.randint(1, S), r.randint(1, S)
    cases.append(f"{n} {m}")
print(t)
print("\n".join(cases))
