#!/usr/bin/env python3
# usage: gen.py seed size; size caps the sum of n (the statement allows 5e5)
import random, sys
r = random.Random(int(sys.argv[1]))
budget = max(1, min(int(sys.argv[2]), 500_000))
sizes = []
while budget >= 1 and len(sizes) < 10_000:
    n = budget if r.random() < 0.3 else r.randint(1, budget)
    sizes.append(n)
    budget -= n
print(len(sizes))
for n in sizes:
    m = r.choice([1, n, r.randint(1, n), r.randint(1, min(n, 5))])
    mode = r.choice(["random", "painted", "ramp"])
    if mode == "random":
        a = [r.randint(1, m) for _ in range(n)]
    elif mode == "ramp":
        a = [i % m + 1 for i in range(n)]
    else:
        a = [r.randint(1, m) for _ in range(n)]
        for _ in range(r.randint(1, 3 * (n // m) + 1)):
            l = r.randint(0, n - m)
            a[l:l + m] = range(1, m + 1)
        for _ in range(r.randint(0, max(1, n // 10))):
            a[r.randrange(n)] = r.randint(1, m)
    print(n, m)
    print(*a)
