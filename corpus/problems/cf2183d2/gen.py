#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # a path from the root: the deepest tree, one vertex per layer
    n = 200_000
    print(1)
    print(n)
    print("\n".join(f"{i} {i + 1}" for i in range(1, n)))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
