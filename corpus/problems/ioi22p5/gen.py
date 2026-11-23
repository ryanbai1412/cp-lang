# usage: gen.py SEED [N]: one case in the sample grader's format (N, then the N hidden types).
import random, sys

r = random.Random(int(sys.argv[1]))
n = int(sys.argv[2]) if len(sys.argv) > 2 else r.choice([2, 3, 6, r.randint(2, 50), r.randint(2, 2000), 2000])
shape = r.choice(["one", "distinct", "few", "many", "rare", "even", "sqrt"])
if shape == "one":
    sizes = [n]
elif shape == "distinct":
    sizes = [1] * n
elif shape == "even":
    k = r.choice([d for d in range(1, n + 1) if n % d == 0])
    sizes = [n // k] * k
else:
    k = {"few": r.randint(1, min(n, 5)), "many": r.randint(1, n), "rare": r.randint(2, min(n, 8)),
         "sqrt": max(1, int(n ** 0.5))}[shape]
    k = min(k, n)
    sizes = [1] * k
    for _ in range(n - k):
        sizes[r.randrange(k)] += 1
    if shape == "rare" and k > 1:
        moved = sizes[0] - r.randint(1, 3)
        if moved > 0:
            sizes[0] -= moved
            sizes[1] += moved
ids = r.sample(range(10**9 + 1), len(sizes)) if r.random() < 0.6 else r.sample(range(len(sizes) + 3), len(sizes))
types = [t for t, s in zip(ids, sizes) for _ in range(s)]
r.shuffle(types)
print(n)
print(*types)
