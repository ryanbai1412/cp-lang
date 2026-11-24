#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 1000 tests at k = 60: odd x (the longest undo chains), the extremes 1 and 2^61 - 1.
    r = random.Random(2138)
    print(1000)
    top = (1 << 61) - 1
    for i in range(1000):
        x = [1, top, 1 << 60][i] if i < 3 else r.randrange(1, top + 1) | 1
        print(60, x)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    k = r.randint(1, 8 if size == 0 else 40)
    x = r.randint(1, (1 << (k + 1)) - 1)
    print(k, x)
