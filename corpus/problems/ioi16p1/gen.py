"""ioi16p1 generator: `n l u` and n weights with u - l >= max w - min w."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = 200_000 if big else (int(sys.argv[2]) if len(sys.argv) > 2 else 8)
n = size if big else r.randint(1, max(1, size))
base = r.choice([1, 10, 10**4, 10**9]) if not big else 10**4
w = [base + r.randint(0, r.choice([0, 3, base])) for _ in range(n)]
d = max(w) - min(w)
cap = 2**31 - 1
l = r.randint(1, max(1, min(cap - d, sum(w) + 5))) if not big else min(cap - d, sum(w) - 3)
u = min(cap, max(max(w), l + d + r.choice([0, 0, r.randint(0, 10)])))
print(n, l, u)
print(*w)
