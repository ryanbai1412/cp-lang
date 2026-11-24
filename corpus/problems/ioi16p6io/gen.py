"""gen.py seed size: IOI 2016 Aliens (stdin). n <= 1e5 points (r, c) in 0..m, m <= 1e6,
k in 1..=n photos. Points spread over the grid or bunched near the diagonal."""
import random, sys
if sys.argv[1] == "--max":
    r = random.Random(0)
    n, m = 100000, 1000000
    print(n, m, n // 3)
    print("\n".join(f"{r.randrange(m)} {r.randrange(m)}" for _ in range(n)))
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(1, max(1, min(100_000, size)))
m = rng.randint(1, max(1, min(10 ** 6, 4 * size)))
k = rng.randint(1, n)
near = rng.random() < 0.5
out = [f"{n} {m} {k}"]
for _ in range(n):
    r = rng.randrange(m)
    c = min(m - 1, max(0, r + rng.randint(-3, 3))) if near else rng.randrange(m)
    out.append(f"{r} {c}")
print("\n".join(out))
