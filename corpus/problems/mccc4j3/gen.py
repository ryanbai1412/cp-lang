"""mccc4j3 generator: n, then n scores in 1..=100; half the cases have a balance point.
--max: n = 1e6, balanced."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 1_000_000 if big else r.randint(1, 1 + size * 3)
if big:
    # Exactly n scores, the right half raised a score at a time until it balances the left.
    left = [r.randint(1, 100) for _ in range(n // 2)]
    right = [r.randint(1, 50) for _ in range(n - n // 2)]
    gap, i = sum(left) - sum(right), 0
    while gap:
        up = min(gap, 100 - right[i]) if gap > 0 else -min(-gap, right[i] - 1)
        right[i] += up
        gap -= up
        i += 1
    a = left + right
elif r.random() < 0.5:
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
