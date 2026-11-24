#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one path of n nodes, k = n / 2: the deepest knapsack chain and the largest rerooting tables
    rng = random.Random(1)
    n = 4000
    print(1)
    print(n, n // 2)
    print(*[rng.randint(1, 10**9) for _ in range(n)])
    print("\n".join(f"{i} {i + 1}" for i in range(1, n)))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, 10)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 10) for _ in range(n)])
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
