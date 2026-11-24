"""ioi17p2 generator: `n m`, then n sorted red and m sorted blue positions, all distinct."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 100_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 8)
n = size if big else r.randint(1, max(1, size))
m = size if big else r.randint(1, max(1, size))
top = 10**9 if big else r.choice([n + m, 3 * (n + m), 10**9])
pos = r.sample(range(top), n + m)
print(n, m)
print(*sorted(pos[:n]))
print(*sorted(pos[n:]))
