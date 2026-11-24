"""ioi94p1 generator: n <= 100, then row i of the triangle (i values in 0..=99). --max: n = 100."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 100 if big else r.randint(1, min(100, size + 1))
print(n)
for i in range(1, n + 1):
    print(*[r.randint(0, 99) for _ in range(i)])
