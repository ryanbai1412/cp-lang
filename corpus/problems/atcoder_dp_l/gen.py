import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "3000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 3000 if size >= 3000 else r.randint(1, min(4 * size, 3000))
top = 10**9 if size >= 3000 else r.choice([5, 10**9])
print(n)
print(*[r.randint(1, top) for _ in range(n)])
