#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5 and k = 1: most drones start at energy 1 (one huge pile climbing
    # past every level), the rest spread over 1..2n.
    rng = random.Random(2157)
    n, k = 200_000, 1
    print(1)
    print(n, k)
    print(*[1 if rng.random() < 0.6 else rng.randint(1, 2 * n) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 5))
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 2 * n) for _ in range(n)])
