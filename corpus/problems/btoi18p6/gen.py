"""btoi18p6 (Paths) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, M, K (N, M <= 3e5 with K <= 4, or N, M <= 1e5 with K = 5), N colours in 1..=K, then M
distinct edges a b, a != b. --max: N = M = 3e5, K = 4 with colours spread evenly, so every
colour subset holds paths at every vertex.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
k = 4 if MAX else rng.randint(1, 5)
cap = 3 * 10**5 if k <= 4 else 10**5
n = cap if MAX else rng.randint(1, max(1, min(cap, int(sys.argv[2]))))
m = cap if MAX else rng.randint(0, min(cap, n * (n - 1) // 2, 3 * n))
colour = [rng.randint(1, k) for _ in range(n)]
edges = set()
while len(edges) < m:
    a, b = rng.randint(1, n), rng.randint(1, n)
    if a != b and (b, a) not in edges:
        edges.add((a, b))
print(n, m, k)
print(*colour)
print("\n".join(f"{a} {b}" for a, b in edges))
