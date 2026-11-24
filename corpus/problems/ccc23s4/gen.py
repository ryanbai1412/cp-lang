#!/usr/bin/env python3
# gen.py SEED SIZE | --max: SIZE 0/1 small; --max is SIZE 2 with seed 0; SIZE >= 2 N = M = 2000 with costs up to 1e9 (total > 2^32).
import random, sys
if sys.argv[1] == "--max": sys.argv[1:] = ["0", "2"]
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S <= 1:
    n = r.randint(2, 4 + 4 * S); m = r.randint(1, 6 + 8 * S); L = r.choice([0, 3, 10]); C = 10
else:
    n = 2000; m = 2000; L = r.choice([0, 5, 10 ** 9]); C = 10 ** 9
out = ["%d %d" % (n, m)]
for i in range(m):
    if S >= 2 and i < n - 1 and r.random() < 0.9:
        u, v = i + 1, i + 2
    else:
        u, v = r.sample(range(1, n + 1), 2)
    c = C if S >= 2 and r.random() < 0.5 else r.randint(1, C)
    out.append("%d %d %d %d" % (u, v, r.randint(0, L), c))
print("\n".join(out))
