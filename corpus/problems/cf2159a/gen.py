#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(2, size * 3))
    arr = [x for x in range(1, n + 1) for _ in range(2)]
    rng.shuffle(arr)
    print(n)
    print(*arr)
