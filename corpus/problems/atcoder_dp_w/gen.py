import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 200000 if size >= 200000 else r.randint(1, size)
m = 200000 if size >= 200000 else r.randint(1, size)
top = 10**9 if size >= 200000 else r.choice([5, 10**9])
print(n, m)
for _ in range(m):
    l = r.randint(1, n)
    print(l, r.randint(l, n), r.randint(-top, top))
