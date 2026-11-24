#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one n = 2e5 test (sum n): the first half alternates +1e9 / -1e9 so the prefix sums tie
    # in huge groups (the index tiebreak decides), the second half climbs by up to 1e9 so the
    # prefixes reach 1e14
    g = random.Random(1)
    n = 200_000
    a = [10**9 if i % 2 == 0 else -10**9 for i in range(n // 2)]
    a += [g.randint(-10**9, 10**9) if g.random() < 0.3 else 10**9 for _ in range(n - n // 2)]
    print(1)
    print(n)
    print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 12)
    print(n)
    print(*[rng.randint(-20, 20) for _ in range(n)])
