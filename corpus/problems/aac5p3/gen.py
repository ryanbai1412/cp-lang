import random
import sys

# `seed size`: a random tree of n <= size breakpoints (`--max`: n = k = 2e5), hill difficulties
# from a small range (ties between siblings) or up to 1e9, k <= size skills.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
big = size >= 200000
n = 200000 if big else r.randint(1, size)
k = 200000 if big else r.randint(1, size)
top = 10**9 if big or r.random() < 0.5 else 12
p = list(range(1, n + 1))
r.shuffle(p)
p.remove(1)
p.insert(0, 1)
print(n, k)
for i in range(1, n):
    lo = 0 if r.random() < 0.5 else max(0, i - 3)
    print(p[r.randint(lo, i - 1)], p[i], r.randint(1, top))
print(*[r.randint(1, top) for _ in range(k)])
