#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = k = 2e5 test (sum n): a shuffled permutation of 0..n-1, so every value below
    # k - 1 is marked and the mex scan runs to the end
    g = random.Random(1)
    n = 200_000
    a = list(range(n))
    g.shuffle(a)
    print(1)
    print(n, n)
    print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    k = rng.randint(2, n)
    print(n, k)
    print(*[rng.randint(0, n) for _ in range(n)])
