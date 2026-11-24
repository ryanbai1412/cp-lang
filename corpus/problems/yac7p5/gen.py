"""yac7p5 hidden data: N (2 <= N <= 1e4) and a uniformly random permutation p of 1..=N."""
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "10000"]
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 10000 if size >= 10000 else rng.randint(2, max(2, size))
p = list(range(1, n + 1))
rng.shuffle(p)
print(n)
print(*p)
