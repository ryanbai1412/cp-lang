"""ioi19p1 generator: n pairs of shoes (sizes may repeat), shuffled; left is -size, right is +size."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n = rng.randint(1, max(1, size))
sizes = [rng.randint(1, rng.choice([1, 2, n])) for _ in range(n)]
s = [x for v in sizes for x in (-v, v)]
rng.shuffle(s)
print(n)
print(*s)
