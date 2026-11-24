#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 5000 and as many distinct frequencies as fit (value v appears v times),
    # so the subset-sum over frequencies is as large as it gets.
    rng = random.Random(2165)
    n = 5000
    a, v = [], 1
    while len(a) + v <= n:
        a += [v] * v
        v += 1
    a += rng.choices(range(1, n + 1), k=n - len(a))
    rng.shuffle(a)
    print(1)
    print(n)
    print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 16)
    print(n)
    print(*[rng.randint(1, n) for _ in range(n)])
