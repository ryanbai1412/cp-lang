#!/usr/bin/env python3
# Plants an array, then keeps only tuples it satisfies, so a valid answer exists.
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, min(100, max(3, size * 4)))
    while True:
        k = rng.randint(1, min(n, rng.choice([2, 4, n])))
        a = [rng.choice([rng.randrange(k), k, k + 1 + rng.randrange(3)]) for _ in range(n)]
        want = rng.randint(1, min(100, max(2, size * 3)))
        ts = []
        for _ in range(4000):
            if len(ts) == want:
                break
            c, l = rng.choice([1, 2]), rng.randint(1, n)
            r = rng.randint(l, min(n, l + rng.choice([2, 8, n])))
            seg = a[l - 1:r]
            m = 0
            while m in seg:
                m += 1
            if (min(seg) if c == 1 else m) == k:
                ts.append((c, l, r))
        if ts:
            break
    print(n, k, len(ts))
    for c, l, r in ts:
        print(c, l, r)
