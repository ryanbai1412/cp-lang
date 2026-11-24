"""noi07p5 generator: `k n` with 2 <= k <= 5 and k <= n <= 1e15."""
import random
import sys

if sys.argv[1] == "--max":
    print(5, 10**15)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
k = rng.randint(2, 5)
print(k, rng.randint(k, max(k, rng.choice([size, size**4, 10**15]))))
