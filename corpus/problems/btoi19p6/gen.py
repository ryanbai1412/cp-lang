"""gen.py SEED SIZE | --max: BOI 2019 Olympiads. n <= 500 contestants, k <= 6 events, scores
0..=1e6, c <= min(2000, C(n, k)). Shapes: small score range (many ties) or full range."""
import math, random, sys

if sys.argv[1] == "--max":
    rng, n, k, c, top = random.Random(0), 500, 6, 2000, 10 ** 6
else:
    rng = random.Random(int(sys.argv[1]))
    size = int(sys.argv[2])
    k = rng.randint(1, 6)
    n = rng.randint(k, max(k, min(500, size)))
    c = rng.randint(1, min(2000, math.comb(n, k)))
    top = rng.choice([3, 10, 10 ** 6])
print(n, k, c)
for _ in range(n):
    print(*(rng.randint(0, top) for _ in range(k)))
