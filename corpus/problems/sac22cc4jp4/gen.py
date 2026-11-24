#!/usr/bin/env python3
# gen.py SEED SIZE: SIZE 0/1 small; SIZE >= 2 n = 1e5 with K up to 1e18.
import random, sys
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "2"]
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
n = r.randint(1, 5 + 5 * S) if S <= 1 else 10 ** 5
k = r.randint(1, 20) if S <= 1 else r.choice([10 ** 18, 10 ** 18 - 1, r.randint(1, 10 ** 18)])
p = list(range(1, n + 1))
if S >= 2 and r.random() < 0.3:
    p = p[1:] + p[:1]
else:
    r.shuffle(p)
print(n, k)
print(*p)
