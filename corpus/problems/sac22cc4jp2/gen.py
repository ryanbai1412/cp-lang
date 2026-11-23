"""sac22cc4jp2 generator: n slots, q commands `set|ask shape i`."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
n, q = rng.randint(1, max(1, size)), rng.randint(1, max(1, 2 * size))
print(n, q)
for _ in range(q):
    print(rng.choice(["set", "ask"]), rng.choice(["square", "circle", "triangle"]), rng.randint(1, n))
