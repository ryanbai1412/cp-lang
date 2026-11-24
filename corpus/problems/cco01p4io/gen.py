#!/usr/bin/env python3
"""Hidden test for CCO 2001 P4: `gen.py SEED SIZE | --max` prints `n d h`: n <= 100 levels, the
drop budget d (the optimal worst case, so it is tight) and the breaking level h in 1..=n.
--max is n = 100."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100 if MAX else r.randint(1, min(100, max(1, int(sys.argv[2]))))
f = [0] * n  # f[s]: fewest drops that always settle s + 1 candidate levels
for s in range(1, n):
    f[s] = min(1 + max(i, f[s - i - 1]) for i in range(s))
print(n, max(1, f[n - 1]), r.randint(1, n))
