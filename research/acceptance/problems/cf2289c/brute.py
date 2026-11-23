import sys
from collections import Counter
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); p = [int(next(it)) for _ in range(n)]
    c = Counter((r - l, min(p[l:r])) for l in range(n) for r in range(l + 1, n + 1))
    print(sum(v == 1 for v in c.values()))
