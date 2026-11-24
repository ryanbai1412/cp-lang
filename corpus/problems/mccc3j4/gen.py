"""mccc3j4 generator: `N T`, then each student's first target in 1..=N."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 2000 if big else r.randint(1, max(1, size))
t = 2000 if big else r.randint(1, max(1, 2 * size))
if not big and r.random() < 0.3:
    first = [1] * n  # everyone throws at one student
else:
    first = [r.randint(1, n) for _ in range(n)]
print(n, t)
print(*first)
