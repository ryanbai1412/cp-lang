#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 500 tests of n = 100 random permutations (no sum bound on n)
    g = random.Random(1)
    print(500)
    for _ in range(500):
        p = list(range(1, 101))
        g.shuffle(p)
        print(100)
        print(*p)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    print(n)
    p = list(range(1, n + 1))
    rng.shuffle(p)
    print(*p)
