#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":  # one case n = r = 5e5 (the port's bounds), a and c up to 1e9
    rng = random.Random(0); n = r = 500000
    print(1); print(n, r)
    print(*[rng.randint(0, 10**9) for _ in range(n)]); print(*[rng.randint(1, 10**9) for _ in range(n)]); sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    r = rng.randint(2, 6)
    print(n, r)
    print(*[rng.randint(0, 20) for _ in range(n)])
    print(*[rng.randint(1, 50) for _ in range(n)])
