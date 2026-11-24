#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # t = 100 tests (no sum bound), each n = 100, a few already sorted
    rng = random.Random(1)
    print(100)
    for i in range(100):
        a = [rng.randint(1, 100) for _ in range(100)]
        if i % 10 == 0:
            a.sort()
        print(100)
        print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(1, 14)
    a = [rng.randint(1, n) for _ in range(n)]
    if rng.random() < 0.3: a.sort()
    print(n)
    print(*a)
