"""ioi19p1 generator: n pairs of shoes (sizes may repeat), left is -size, right is +size.

Shapes: shuffled; or every right shoe first and the lefts in reverse, the most swaps."""
import random
import sys
if sys.argv[1] == "--max":
    n = 100000
    sizes = list(range(1, n + 1))
    print(n); print(*(sizes + [-v for v in reversed(sizes)]))
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
sizes = [rng.randint(1, rng.choice([1, min(2, n), n])) for _ in range(n)]
if rng.random() < 0.2:
    s = sizes + [-v for v in reversed(sizes)]
else:
    s = [x for v in sizes for x in (-v, v)]
    rng.shuffle(s)
print(n)
print(*s)
