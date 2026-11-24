#!/usr/bin/env python3
"""gen.py seed size | --max: CCO 2017 Rainfall Capture. n <= 500 pillars of heights 1..=50.
--max: n = 500 with every height present and many tall pillars, so the reachable volumes run
up to the largest the bound allows."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 500 if MAX else max(1, min(500, int(sys.argv[2])))
h = (list(range(1, 51)) + [r.choice([50, 49, r.randint(1, 50)]) for _ in range(450)]) if MAX else \
    [r.randint(1, 50) for _ in range(n)]
r.shuffle(h)
print(n)
print(*h)
