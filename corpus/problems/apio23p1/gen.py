# usage: gen.py SEED [SUM_N]: T cases in the sample grader's format, sum N and sum M <= SUM_N.
import random, sys

r = random.Random(int(sys.argv[1]))
budget = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([6, 20, 200, 2000])
cases = []
left_n = left_m = budget
while left_n >= 2 and len(cases) < 1 + r.randint(0, 4):
    n = r.randint(2, max(2, min(left_n, budget // r.randint(1, 3))))
    left_n -= n
    shape = r.choice(["tree", "path", "sparse", "dense"])
    if shape == "path":
        pairs = [(i, i + 1) for i in range(n - 1)]
    else:
        pairs = [(r.randrange(i), i) for i in range(1, n)] if shape == "tree" else []
        want = min(left_m, {"tree": 0, "sparse": n + n // 2, "dense": n * (n - 1) // 2}[shape])
        seen = set(pairs)
        for _ in range(4 * want):
            if len(pairs) >= want:
                break
            u, v = sorted(r.sample(range(n), 2))
            if (u, v) not in seen:
                seen.add((u, v))
                pairs.append((u, v))
    r.shuffle(pairs)
    if r.random() < 0.15 and len(pairs) > 1:
        pairs.pop()  # may cut off H
    pairs = pairs[:left_m]
    left_m -= len(pairs)
    h = r.randint(1, n - 1)
    weights = r.choice([(1, 10), (1, 1000), (1, 10**9), (10**9 - 10, 10**9)])
    kinds = r.choice([(1,), (0, 1), (1, 2), (0, 1, 2), (1, 1, 2), (0, 1, 1, 2, 2)])
    arr = [r.choice(kinds) for _ in range(n)]
    arr[0] = arr[h] = 1
    k = r.choice([1, 2, 30, 70, 10**6, r.randint(1, 10**6)])
    cases.append((n, k, h, arr, pairs, weights))
print(len(cases))
for n, k, h, arr, pairs, (lo, hi) in cases:
    print(n, len(pairs), k)
    print(h)
    print(*arr)
    for u, v in pairs:
        if r.random() < 0.5:
            u, v = v, u
        print(u, v, r.randint(lo, hi))
