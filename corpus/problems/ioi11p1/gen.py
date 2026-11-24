#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; `n m p q`, m trails (every fountain on one), q lengths.
import random, sys
MAX = sys.argv[1] == "--max"  # n = m = 150000, q = 2000
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 150000 if MAX else r.randint(2, 7)
extra = 0 if MAX else r.randint(0, 5)
trails = [(i, r.choice([j for j in range(n) if j != i]) if not MAX else (i + r.randint(1, n - 1)) % n)
          for i in range(n)]
for _ in range(extra):
    a, b = r.sample(range(n), 2); trails.append((a, b))
r.shuffle(trails)
q = 2000 if MAX else r.randint(1, 6)
print(n, len(trails), r.randrange(n), q)
for a, b in trails: print(a, b)
print(*[r.choice([r.randint(1, 3 * n), r.randint(1, 10**9)]) for _ in range(q)])
