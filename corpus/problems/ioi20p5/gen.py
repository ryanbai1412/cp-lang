"""ioi20p5 generator: n, then the hidden species s[0..n) (0 = A, 1 = B, s[0] = 0)."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 50
n = 20_000 if big else r.randint(2, max(2, size))
bias = r.random()
print(n)
print(0, *(int(r.random() < bias) for _ in range(n - 1)))
