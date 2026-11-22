#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    q = rng.randint(2, max(4, size * 6))
    print(q)
    cur_len = 0
    for _ in range(q):
        if cur_len == 0:
            op = 2
        elif cur_len == 1:
            op = rng.choice([2, 3])
        else:
            op = rng.choice([1, 2, 3])
        if op == 1:
            cur_len -= 1
            print(1)
        elif op == 2:
            cur_len = 2 * cur_len + 1
            print(2, rng.randint(1, 1000000006))
        else:
            print(3)
