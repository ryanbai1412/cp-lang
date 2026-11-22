#!/usr/bin/env python3
import random, sys
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
