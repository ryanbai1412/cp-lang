#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 7 if size == 0 else 18)
    single = r.randint(1, n)
    arr = [single]
    for x in range(1, n + 1):
        if x != single:
            arr.extend([x, x])
    r.shuffle(arr)
    print(n)
    print(*arr)
