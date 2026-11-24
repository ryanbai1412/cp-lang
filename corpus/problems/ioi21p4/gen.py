"""ioi21p4 generator: n q, strings a and b over ACT (b often a shuffle of a, so more queries
are possible; --max rotates aligned triples), then q queries x y with 0 <= x <= y < n."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 100_000 if big else r.randint(1, max(1, size))
q = 100_000 if big else r.randint(1, max(1, 3 * size))
a = [r.choice("ACT") for _ in range(n)]
b = a[:]
if big:
    # rotate each aligned block of three, so block-aligned queries are always possible
    for i in range(0, n - 2, 3):
        b[i:i + 3] = b[i + 1:i + 3] + b[i:i + 1]
for _ in range(0 if big else r.randint(0, n)):
    i, j = r.randrange(n), r.randrange(n)
    b[i], b[j] = b[j], b[i]
if not big and r.random() < 0.3:
    b[r.randrange(n)] = r.choice("ACT")
print(n, q)
print("".join(a))
print("".join(b))
for _ in range(q):
    if big and r.random() < 0.5:
        x = 3 * r.randrange(n // 3)
        y = x + 3 * r.randint(1, (n - x) // 3) - 1
    else:
        x = r.randrange(n)
        y = r.randrange(x, n) if big or r.random() < 0.5 else min(n - 1, x + r.randint(0, 3))
    print(x, y)
