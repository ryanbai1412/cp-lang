"""joi16op3 generator: N L, then N distinct heights in 1..=1000. --max: N = 100, L = 1000."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 100 if big else r.randint(1, min(100, size + 2))
lim = 1000 if big else r.randint(1, min(1000, 1 + size * r.choice([1, 5, 50])))
top = 1000 if big else r.choice([n + 3, 4 * n, 1000])
print(n, lim)
print(*r.sample(range(1, top + 1), n))
