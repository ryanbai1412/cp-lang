import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 100000:
    k, n = 100000, 100
else:
    k = r.randint(1, 10 * size)
    n = r.randint(1, min(100, k, size + 1))
print(n, k)
print(*sorted(r.sample(range(1, k + 1), n)))
