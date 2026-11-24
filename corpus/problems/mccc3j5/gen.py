"""mccc3j5 generator: N M, then M distinct restricted pairs. --max: N = 18, M = 153."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 18 if big else r.randint(1, min(18, size + 2))
pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
m = len(pairs) if big else r.randint(0, len(pairs))
print(n, m)
for u, v in r.sample(pairs, m):
    print(*(r.sample([u, v], 2)))
