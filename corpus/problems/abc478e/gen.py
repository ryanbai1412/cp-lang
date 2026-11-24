"""abc478e generator: `n q`, then q conditions `t u v` (A[u] <= A[v] for t = 0, < for t = 1)."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n, q = random.Random(0), 200000, 200000
    print(n, q)
    for i in range(q):
        u = rng.randint(1, n - 1)
        print(rng.randint(0, 1), u, rng.randint(u + 1, n))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
q = rng.randint(1, max(1, size))
order = list(range(1, n + 1))
rng.shuffle(order)
acyclic = rng.random() < 0.6
print(n, q)
for _ in range(q):
    i, j = rng.randrange(n), rng.randrange(n)
    if acyclic and i > j:
        i, j = j, i
    t = 0 if acyclic and i == j else rng.choice([0, 0, 1])
    print(t, order[i], order[j])
