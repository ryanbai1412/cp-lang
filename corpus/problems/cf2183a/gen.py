#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 100 tests of n = 100: both ends 0, one end 1, all ones, all zeros but the middle
    g = random.Random(1)
    print(100)
    for i in range(100):
        a = [g.randint(0, 1) for _ in range(100)]
        if i % 4 == 0:
            a[0] = a[-1] = 0
        elif i % 4 == 1:
            a[g.choice((0, -1))] = 1
        elif i % 4 == 2:
            a = [1] * 100
        else:
            a = [0] * 100
            a[50] = 1
        print(100)
        print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(3, 14)
    print(n)
    print(*[rng.randint(0, 1) for _ in range(n)])
