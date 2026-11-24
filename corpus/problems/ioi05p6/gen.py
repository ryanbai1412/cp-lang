#!/usr/bin/env python3
"""gen.py seed size | --max: IOI 2005 Rivers. n <= 100 villages, k <= 50 mills, each village
"w v d" (w <= 1e4 trees, flows to v, d <= 1e4 km), all draining to 0. --max: n = 100, k = 50,
a deep river (mostly a path, a few side streams) with the most trees and longest legs."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 100 if MAX else max(2, min(100, int(sys.argv[2])))
k = 50 if MAX else r.randint(1, min(50, n))
print(n, k)
for v in range(1, n + 1):
    down = v - 1 if MAX and r.random() < 0.85 else r.randint(0, v - 1)
    print(r.randint(9_000, 10_000) if MAX else r.randint(0, 10_000), down,
          r.randint(9_000, 10_000) if MAX else r.randint(1, 10_000))
