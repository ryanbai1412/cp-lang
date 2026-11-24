"""joi16fp2 generator: N M K, A B C (B < C < A), T, then the M express stations (1 and N
included, increasing). --max: N = 1e9, M = K = 3000 sized stops, T = 1e18."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
if big:
    n, m, k = 10**9, 3000, 3000

else:
    n = r.randint(2, 4 + size * size * 3)
    m = r.randint(2, min(n, 2 + size))
    k = r.randint(m, min(3000, m + 2 * size + 1))
b = r.randint(1, 5 if not big else 10**6)
c = r.randint(b + 1, b + 6 if not big else 10**8)
a = r.randint(c + 1, c + 6 if not big else 10**9)
t = r.randint(1, max(1, n * a // r.choice([1, 2, 4])))
mid = sorted(r.sample(range(2, n), m - 2)) if m > 2 else []
print(n, m, k)
print(a, b, c)
print(t)
for s in [1] + mid + [n]:
    print(s)
