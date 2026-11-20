#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 2)
print(t)
for _ in range(t):
    n = rng.randint(2, max(2, min(7, size + 1)))
    m = rng.randint(1, n * (2 * n - 1))
    perm = list(range(1, n * n + 1))
    rng.shuffle(perm)
    print(n, m)
    for i in range(n):
        print(*perm[i * n:(i + 1) * n])
    for _ in range(n):
        moves = ["D"] * (n - 1) + ["R"] * (n - 1)
        rng.shuffle(moves)
        print("".join(moves))
