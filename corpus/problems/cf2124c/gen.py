#!/usr/bin/env python3
import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 6e5: a divisibility chain that doubles or triples now and then, times
    # x = 210 on a random half, so b reaches near 1e9 and many adjacent pairs constrain x
    rng = random.Random(1)
    n, x = 600_000, 210
    steps = sorted(rng.sample(range(1, n), 21))
    a, v = [], 1
    for i in range(n):
        if steps and steps[0] == i:
            steps.pop(0)
            v *= 2
        a.append(v)
    print(1)
    print(n)
    print(*[w * (x if rng.random() < 0.5 else 1) for w in a])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 6)
print(t)
for _ in range(t):
    n = rng.randint(2, 12)
    x = rng.randint(1, 12)
    a = [rng.randint(1, 6)]
    for _ in range(n - 1):
        a.append(a[-1] * rng.randint(1, 3))
    b = [v * (x if rng.random() < 0.5 else 1) for v in a]
    print(n)
    print(*b)
