"""gen.py seed size: YAC7 P3. 1 <= n <= 1e6, P_i in 1..=n (self-loops allowed).
Shapes: random map, one long cycle, a permutation, long tails into small cycles."""
import random, sys

if "--max" in sys.argv[1:]:
    rng = random.Random(7)
    n, shape = 10 ** 6, "tails"
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2])
    rng = random.Random(seed)
    n = rng.randint(1, max(1, size))
    shape = rng.choice(["random", "cycle", "perm", "tails"])
if shape == "random":
    p = [rng.randint(1, n) for _ in range(n)]
elif shape == "cycle":
    order = list(range(1, n + 1)); rng.shuffle(order)
    p = [0] * n
    for i, v in enumerate(order):
        p[v - 1] = order[(i + 1) % n]
elif shape == "perm":
    p = list(range(1, n + 1)); rng.shuffle(p)
else:
    order = list(range(1, n + 1)); rng.shuffle(order)
    k = rng.randint(1, max(1, n // 100))
    p = [0] * n
    for i, v in enumerate(order):
        p[v - 1] = order[(i + 1) % k] if i < k else order[i - 1]
print(n)
print(*p)
