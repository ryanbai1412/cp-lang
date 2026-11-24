"""ceoi17p2 generator: n pairs of odds `a b`, each in 1.0..=1000.0 with up to 4 decimals."""
import random
import sys
if sys.argv[1:] == ["--max"]:
    # n = 1e5 bookmakers, odds over the whole 1.0..=1000.0 range with 4 decimals
    g = random.Random(1)
    print(10**5)
    for _ in range(10**5): print(f"{g.uniform(1, 1000):.4f} {g.uniform(1, 1000):.4f}")
    sys.exit()


seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
hi = rng.choice([3, 20, 1000])
print(n)
for _ in range(n):
    print(*(f"{rng.uniform(1, hi):.{rng.choice([1, 4])}f}" for _ in range(2)))
