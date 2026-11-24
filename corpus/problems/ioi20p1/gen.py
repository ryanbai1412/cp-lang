"""ioi20p1 (Comparing Plants): `gen.py SEED SIZE`, or `gen.py --max`.

"n k q", then r[0..n-1], then q lines "x y" (2 <= k <= n <= 2e5, 1 <= q <= 2e5,
0 <= x < y < n); r comes from a hidden height order, so it is always consistent. The
output is each compare_plants. --max: n = q = 2e5, k = 1000, random heights.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, k, q = 200_000, 1_000, 200_000
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(2, max(2, min(200_000, size)))
    k = rng.choice([2, n, rng.randint(2, n)])
    q = rng.randint(1, min(200_000, max(1, size)))
h = list(range(n))
rng.shuffle(h)
r = [0] * n
if k <= 64 or not MAX:
    for i in range(n):
        r[i] = sum(h[(i + j) % n] > h[i] for j in range(1, k))
else:
    import bisect
    win = sorted(h[j % n] for j in range(1, k))
    for i in range(n):
        r[i] = len(win) - bisect.bisect_right(win, h[i])
        win.pop(bisect.bisect_left(win, h[(i + 1) % n]))
        bisect.insort(win, h[(i + k) % n])
qs = []
for _ in range(q):
    x, y = sorted(rng.sample(range(n), 2))
    qs.append(f"{x} {y}")
print(n, k, q)
print(*r)
print("\n".join(qs))
