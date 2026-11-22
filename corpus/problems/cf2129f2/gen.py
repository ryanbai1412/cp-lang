#!/usr/bin/env python3
import random, sys

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = 3 if size == 0 else 2
print(T)
for _ in range(T):
    n = r.randint(2, 12 if size == 0 else min(890, 30 * size + 20))
    mode = r.choice(["rand", "rand", "asc", "desc"])
    p = list(range(1, n + 1))
    if mode == "desc":
        p.reverse()
    elif mode == "rand":
        r.shuffle(p)
    print(n)
    print(*p)
