#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5 that has an answer: one group of size 1e5, groups of every small
    # size and many singletons, shuffled.
    r = random.Random(2137)
    n = 200_000
    b = [100_000] * 100_000
    k = 1
    while len(b) + k <= n - 1000:
        b += [k] * k
        k = k % 300 + 1
    b += [1] * (n - len(b))
    r.shuffle(b)
    print(1)
    print(n)
    print(*b)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    if r.random() < 0.5:
        b = []
        rem = n
        while rem > 0:
            k = r.randint(1, rem)
            b.extend([k] * k)
            rem -= k
        r.shuffle(b)
    else:
        b = [r.randint(1, n) for _ in range(n)]
    print(n)
    print(*b)
