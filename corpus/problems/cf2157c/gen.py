#!/usr/bin/env python3
# Plants an array, then keeps only tuples it satisfies, so a valid answer exists.
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 500 tests (no sum bounds) of n = q = 100, k across 1..=n, each with 100 tuples that a
    # planted array satisfies, both kinds mixed and many ranges overlapping.
    rng = random.Random(2157)
    print(500)
    for t in range(500):
        n = 100
        k = [1, 2, n // 2, n][t % 4] if t < 8 else rng.randint(1, n)
        # big run, a permutation of 0..k-1, big run; k + 1 next to the permutation and some k
        # farther out, so mex ranges stretch over the k + 1 values and min ranges sit outside.
        x = rng.randint(0, n - k)
        left, right = x, n - k - x
        a = [k if i < left // 2 or rng.random() < 0.2 else k + 1 for i in range(left)]
        a += rng.sample(range(k), k)
        a += [k if i >= (right + 1) // 2 or rng.random() < 0.2 else k + 1 for i in range(right)]
        ts = []
        while len(ts) < 100:
            c = rng.choice([1, 2]) if k < n else 2
            if c == 1:
                lo, hi = (1, left) if left and (not right or rng.random() < 0.5) else (n - right + 1, n)
                if hi < lo:
                    c, lo, hi = 2, 1, n
            if c == 1:
                l = rng.randint(lo, hi)
                r = rng.randint(l, hi)
            else:
                l = rng.randint(1, left + 1)
                r = rng.randint(left + k, n)
            seg = a[l - 1:r]
            if c == 1:
                ok = min(seg) == k
            else:
                have = set(seg)
                ok = k not in have and all(v in have for v in range(k))
            if ok:
                ts.append((c, l, r))
        print(n, k, len(ts))
        for c, l, r in ts:
            print(c, l, r)
    sys.exit()
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
