import random
import sys

# `seed size`: an n x m grid with n * m <= max(1, 4 size) cells of '.' and '#'
# (`--max`: 1000 x 1000, about half '#').
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
cells = 10**6 if MAX else r.randint(1, max(1, 4 * size))
n = 1000 if MAX else r.randint(1, cells)
m = 1000 if MAX else max(1, cells // n)
p = 0.5 if MAX else r.choice([0.2, 0.4, 0.6])
print(n, m)
for _ in range(n):
    print("".join("#" if r.random() < p else "." for _ in range(m)))
