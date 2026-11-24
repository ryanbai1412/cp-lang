#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":  # one case n = q = 2e5 (both sums), |a_i|, |x| <= 1e8
    rng = random.Random(0); n = q = 200000; L = 10**8
    print(1); print(n, q)
    print(*[rng.choice([rng.randint(-L, L), L, -L]) for _ in range(n)])
    print(*[rng.randint(-L, L) for _ in range(q)]); sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    q = rng.randint(1, 10)
    print(n, q)
    print(*[rng.randint(-20, 20) for _ in range(n)])
    print(*[rng.randint(-20, 20) for _ in range(q)])
