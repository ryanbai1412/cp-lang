#!/usr/bin/env python3
# usage: gen.py seed size; n = min(size, 1e6) cities, fans >= 1 each and 2e9 in total at most
import random, sys
if sys.argv[1] == "--max":  # the largest case the bounds allow
    sys.argv[1:] = ["1", "1000000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = min(size, 10**6) if size > 50 else r.randint(1, max(1, size))
total = 2 * 10**9
mode = r.choice(["ones", "small", "spread", "huge", "max"])
if mode == "ones":
    p = [1] * n
elif mode == "small":
    p = [r.randint(1, 10) for _ in range(n)]
elif mode == "spread":
    p = [r.randint(1, total // n) for _ in range(n)]
elif mode == "huge":
    p = [1] * n
    p[r.randrange(n)] = total - (n - 1)
else:  # the total is exactly 2e9, split at random
    cuts = sorted(r.sample(range(1, total), n - 1))
    p = [b - a for a, b in zip([0] + cuts, cuts + [total])]
shape = r.choice(["random", "path", "star", "caterpillar", "deep"])
par = [0] * n
for i in range(1, n):
    if shape == "path":
        par[i] = i - 1
    elif shape == "star":
        par[i] = 0
    elif shape == "caterpillar":
        par[i] = i - 1 if i <= n // 2 else r.randint(0, n // 2)
    elif shape == "deep":
        par[i] = r.randint(max(0, i - 3), i - 1)
    else:
        par[i] = r.randint(0, i - 1)
perm = list(range(n))
r.shuffle(perm)
q = [0] * n
for i in range(n):
    q[perm[i]] = p[i]
print(n)
print("\n".join(map(str, q)))
edges = [(perm[par[i]], perm[i]) if r.random() < 0.5 else (perm[i], perm[par[i]]) for i in range(1, n)]
r.shuffle(edges)
print("\n".join(f"{a} {b}" for a, b in edges))
