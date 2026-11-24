"""sdoi17r1p2 generator: a tree on n <= 1e5 vertices and m <= 1e5 ops `1 x`, `2 x y`, `3 x`."""
import random
import sys

big = sys.argv[1:] == ["--max"]  # n = m = 1e5, a random tree
if big:
    sys.argv[1:] = ["1", "100000"]
seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 10
rng = random.Random(seed)
n = 100_000 if big else rng.randint(1, max(1, min(size, 100_000)))
m = 100_000 if big else rng.randint(1, max(1, min(size, 100_000)))
shape = "random" if big else rng.choice(["random", "path", "star", "caterpillar"])
print(n, m)
for v in range(2, n + 1):
    if shape == "path":
        p = v - 1
    elif shape == "star":
        p = 1
    elif shape == "caterpillar":
        p = v - 1 if v % 2 == 0 else max(1, v - 2)
    else:
        p = rng.randint(max(1, v - rng.choice([1, 3, v])), v - 1)
    print(*((p, v) if rng.random() < 0.5 else (v, p)))
for _ in range(m):
    k = rng.choice([1, 1, 2, 3])
    x = rng.randint(1, n)
    if k == 2:
        print(2, x, rng.randint(1, n))
    else:
        print(k, x)
