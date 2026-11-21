#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 30)
    print(n)
    for _ in range(n):
        a = r.randint(1, 2 * n - 1)
        b = r.randint(a + 1, 2 * n)
        print(a, b)
