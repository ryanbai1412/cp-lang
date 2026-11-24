"""nccc10s1 generator: n k, then n rows of k T/F answers. --max: n = 1000, k = 10."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 1000 if big else r.randint(1, 1 + size * 2)
k = 10 if big else r.randint(1, min(10, size + 1))
print(n, k)
for _ in range(n):
    print("".join(r.choice("TF") for _ in range(k)))
