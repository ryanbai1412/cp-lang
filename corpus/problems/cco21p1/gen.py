import random
import sys

# `seed size`: n <= 1 + 3 size values in 1..k (k <= 2 + size), q <= 1 + 3 size swaps.
# `--max`: n = 1e5, k = 300, q = 1e6, so many distinct pairs repeat.
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
n = 10**5 if MAX else r.randint(1, 1 + 3 * size)
k = 300 if MAX else r.randint(2, 2 + size)
q = 10**6 if MAX else r.randint(1, 1 + 3 * size)
print(n, k, q)
print(*(r.randint(1, k) for _ in range(n)))
print("\n".join(str(r.randint(1, k - 1)) for _ in range(q)))
