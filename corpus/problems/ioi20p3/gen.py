"""ioi20p3 generator: `n m k` (n even, k <= m), then n rows of m sorted ticket values."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 6
n = 1500 if big else 2 * r.randint(1, max(1, size // 2))
m = 1500 if big else r.randint(1, max(1, size))
k = m if big else r.randint(1, m)
top = 10**9 if big or r.random() < 0.5 else r.randint(0, 5)
print(n, m, k)
for _ in range(n):
    print(*sorted(r.randint(0, top) for _ in range(m)))
