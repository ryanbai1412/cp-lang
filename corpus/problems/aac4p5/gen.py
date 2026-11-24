import random
import sys

# `seed size`: n, m, q <= size (`--max`: n = m = q = 5e5), 1 <= x <= n, q random pairs.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
big = size >= 500000
n = 500000 if big else r.randint(1, size)
q = 500000 if big else r.randint(0, size)
x = r.randint(1, n) if r.random() < 0.5 else r.randint(1, min(n, 3))
m = 500000 if big else r.choice([r.randint(0, size), r.randint(0, n * n), n])
m = min(m, 500000)
print(n, m, x, q)
for _ in range(q):
    print(r.randint(1, n), r.randint(1, n))
