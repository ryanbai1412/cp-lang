import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
if size >= 100000:
    n, k = 100, 100000
    caps = [r.randint(k // 2, k) for _ in range(n)]
else:
    n, k = r.randint(1, min(size + 1, 100)), r.randint(0, 6 * size)
    caps = [r.randint(0, k) for _ in range(n)]
print(n, k)
print(*caps)
