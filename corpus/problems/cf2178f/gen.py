#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n: the path 1-2-..-n, 2e5 deep, every other subtree even
    n = 200000
    print("\n".join(["1", str(n)] + [f"{i} {i + 1}" for i in range(1, n)]))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 16)
    print(n)
    for i in range(2, n + 1):
        print(rng.randint(1, i - 1), i)
