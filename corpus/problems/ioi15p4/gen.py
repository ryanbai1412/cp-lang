"""ioi15p4 generator: n, x, y, m, then m updates `kind year value` (kind 1 sets x, 2 sets y)."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 500_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 8)
n = size if big else r.randint(1, max(1, size))
m = 100_000 if big else r.randint(1, max(1, size))
top = 10**9 if big else r.choice([2, 3, 10, 10**9])
ones = 0.0 if big else r.choice([0.0, 0.5, 0.9])
gx = lambda: 1 if r.random() < ones else r.randint(1, 2 if big else top)
print(n)
print(*(gx() for _ in range(n)))
print(*(r.randint(1, top) for _ in range(n)))
print(m)
for _ in range(m):
    kind = r.randint(1, 2)
    print(kind, r.randrange(n), gx() if kind == 1 else r.randint(1, top))
