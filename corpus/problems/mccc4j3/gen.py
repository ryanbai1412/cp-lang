"""mccc4j3 generator: n, then n scores in 1..=100; half the cases have a balance point.
--max: n = 1e6, balanced."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 999_000 if big else r.randint(1, 1 + size * 3)
if big or r.random() < 0.5:
    left = [r.randint(1, 100) for _ in range(n // 2)] or [r.randint(1, 100)]
    right = [r.randint(1, 100) for _ in range(n - len(left))] or [r.randint(1, 100)]
    while sum(left) != sum(right):
        side = left if sum(left) < sum(right) else right
        side.append(min(100, abs(sum(left) - sum(right))))
    a = left + right
else:
    a = [r.randint(1, 100) for _ in range(n)]
print(len(a))
print(*a)
