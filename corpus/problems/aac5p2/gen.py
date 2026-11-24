import random
import sys

# `seed size`: a hidden permutation of 1..=n, 3 <= n <= max(3, size) (`--max`: n = 1e5).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 100000 if size >= 100000 else r.randint(3, max(3, size))
p = list(range(1, n + 1))
r.shuffle(p)
print(n)
print(*p)
