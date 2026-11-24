#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # one test at n = 2e5 (sum n <= 2e5): a path 1e5 deep (x_i = i), then valid random walks of
    # depth (drops and climbs by one), so the DFS is as deep as it gets and parents keep branching
    rng = random.Random(0)
    n = 200_000
    x = list(range(1, n // 2 + 1))
    while len(x) < n:
        x.append(rng.randint(2, x[-1] + 1) if rng.random() < 0.3 else min(x[-1] + 1, len(x) + 1))
    print(1); print(n); print(*x)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 14)
    x = [1]
    for _ in range(n - 1):
        if rng.random() < 0.85:
            x.append(rng.randint(2, min(n, x[-1] + 1)))
        else:
            x.append(rng.randint(1, n))
    print(n)
    print(*x)
