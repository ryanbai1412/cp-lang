import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "300"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = 300 if size >= 300 else r.randint(1, size)
top = 3 if size >= 300 else r.randint(1, 3)
print(n)
print(*(3 if size >= 300 and i % 2 else r.randint(1, top) for i in range(n)))
