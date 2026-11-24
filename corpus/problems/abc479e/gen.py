"""abc479e generator: an odd n, then n - 1 edges of a random tree."""
import random
import sys

if sys.argv[1] == "--max":
    rng, n = random.Random(0), 199999
else:
    seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rng = random.Random(seed)
    n = 2 * rng.randint(1, max(1, size // 2)) + 1
label = list(range(1, n + 1))
rng.shuffle(label)
print(n)
chain = rng.random() < 0.2
for v in range(1, n):
    p = v - 1 if chain else rng.randrange(v)
    a, b = label[p], label[v]
    print(*((a, b) if rng.random() < 0.5 else (b, a)))
