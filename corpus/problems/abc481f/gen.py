"""abc481f generator: `n m`, then m arcs `u v w` (digit w in 1..9), every vertex reachable from 1."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n, m, digits = random.Random(0), 200000, 200000, 2
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rng = random.Random(seed)
    n = rng.randint(2, max(2, size))
    m = rng.randint(n - 1, 3 * n)
    digits = rng.choice([1, 2, 9])
arcs = [(rng.randint(1, v - 1), v) for v in range(2, n + 1)]
arcs += [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m - len(arcs))]
rng.shuffle(arcs)
print(n, len(arcs))
for u, v in arcs:
    print(u, v, rng.randint(1, digits))
