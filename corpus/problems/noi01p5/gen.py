import random
import sys

# `seed size`: n <= 6 terms, M <= max(1, size) (`--max`: n = 6, M = 150, every p = 3).
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 150 if MAX else int(sys.argv[2])
n = 6 if MAX else r.randint(1, 6)
m = 150 if MAX else r.randint(1, max(1, min(150, size)))
print(n)
print(m)
for _ in range(n):
    print(r.choice([-1, 1]) * r.randint(1, 20) if MAX else r.randint(-20, 20), 3 if MAX else r.randint(1, 3))
