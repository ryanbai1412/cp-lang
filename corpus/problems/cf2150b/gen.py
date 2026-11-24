#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5 whose row counts come from a valid grid, so the answer is nonzero
    # and every row takes a binomial.
    rng = random.Random(2150)
    n = 200_000
    a = [0] * n
    for i in range(n):
        a[rng.randint(0, min(i, n - 1 - i))] += 1
    print(1)
    print(n)
    print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(3, size * 5))
    print(n)
    if rng.random() < 0.7:
        # Generate a valid-sum array
        a = [0] * n
        for i in range(n):
            row = rng.randint(0, min(i, n - 1 - i))
            a[row] += 1
        print(*a)
    else:
        print(*[rng.randint(0, 3) for _ in range(n)])
