#!/usr/bin/env python3
import random
import sys

seed = int(sys.argv[1])
size = int(sys.argv[2])
rng = random.Random(seed)

if size >= 10000:
    t = 1
    n = size
    q = size
    print(t)
    print(n, q)
    alphabet = "ab" if seed % 3 == 0 else "abc" if seed % 3 == 1 else "abcde"
    s = "".join(rng.choice(alphabet) for _ in range(n))
    print(s)
    for _ in range(q):
        l = rng.randint(1, n - 1)
        r = rng.randint(l + 1, n)
        print(l, r)
else:
    t = rng.randint(1, 4)
    print(t)
    for _ in range(t):
        n = rng.randint(2, max(2, size))
        q = rng.randint(1, max(2, size))
        print(n, q)
        mode = rng.randint(0, 3)
        if mode == 0:
            s = "a" * n
        elif mode == 1:
            s = ("ab" * ((n + 1) // 2))[:n]
        elif mode == 2:
            s = "".join(rng.choice("ab") for _ in range(n))
        else:
            s = "".join(rng.choice("abcde") for _ in range(n))
        print(s)
        for _ in range(q):
            l = rng.randint(1, n - 1)
            r = rng.randint(l + 1, n)
            print(l, r)
