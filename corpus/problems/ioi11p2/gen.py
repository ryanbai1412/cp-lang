"""ioi11p2 (Race) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N K" (1 <= N <= 2e5, 1 <= K <= 1e6), then N - 1 roads "u v len" (cities 0..N-1, a tree,
0 <= len <= 1e6).
--max: N = 2e5, K = 1e6, a long spine with random branches and short roads, so the
small-to-large maps hold many distances up to K.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, k, top = 200_000, 1_000_000, 20
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(200_000, size))
    top = rng.choice([0, 3, 1_000_000])
    k = rng.randint(1, max(1, min(1_000_000, rng.choice([top * 4, 1_000_000]))))
names = list(range(n))
rng.shuffle(names)
roads = []
for v in range(1, n):
    u = v - 1 if (MAX and rng.random() < 0.5) or rng.random() < 0.3 else rng.randrange(v)
    a, b = names[u], names[v]
    roads.append((a, b) if rng.random() < 0.5 else (b, a))
rng.shuffle(roads)
print(n, k)
print("\n".join(f"{a} {b} {rng.randint(0, top)}" for a, b in roads))
