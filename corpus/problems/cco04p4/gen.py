"""cco04p4 generator: two walkers, each `x y n` and n moves `d axis` with d != 0."""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)
for _ in range(2):
    n = rng.randint(1, max(1, min(size, 6)))
    print(rng.randint(-20, 20), rng.randint(-20, 20), n)
    for _ in range(n):
        print(rng.choice([-1, 1]) * rng.randint(1, 5), rng.choice("XY"))
