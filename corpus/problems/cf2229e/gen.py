#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n: a path under random labels, 2e5 deep from the root n
    g = random.Random(1)
    n = 200000
    perm = list(range(1, n + 1))
    g.shuffle(perm)
    print("\n".join(["1", str(n)] + [f"{perm[i]} {perm[i + 1]}" for i in range(n - 1)]))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    print(n)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    for i in range(1, n):
        p = rng.randint(0, i - 1)
        print(perm[p], perm[i])
