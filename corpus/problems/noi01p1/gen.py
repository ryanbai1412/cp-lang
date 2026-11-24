import random
import sys

# `seed size`: n <= max(1, size) animals, k <= 2 size statements (`--max`: n = 5e4, k = 1e5).
# A hidden kind per animal; about a third of the statements lie (wrong relation, an animal
# past n, or x eats x).
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
n = 50_000 if MAX else r.randint(1, max(1, size))
k = 100_000 if MAX else r.randint(0, 2 * size)
kind = [r.randrange(3) for _ in range(n + 1)]
print(n, k)
for _ in range(k):
    x, y = r.randint(1, n), r.randint(1, n)
    claim = 1 if kind[x] == kind[y] else 2 if (kind[x] - kind[y]) % 3 == 1 else r.randint(1, 2)
    roll = r.random()
    if roll < 0.15:
        claim = 3 - claim
    elif roll < 0.22:
        x = n + r.randint(1, 10)
    elif roll < 0.27:
        claim, y = 2, x
    print(claim, x, y)
