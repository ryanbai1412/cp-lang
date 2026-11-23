import sys
from itertools import combinations
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, m, q = int(next(it)), int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    for _ in range(q):
        i, v = int(next(it)), int(next(it)); b = a[:]; b[i - 1] += v; e = b[::-1]
        cnt = 0
        for cuts in combinations(range(1, n), m - 1):
            bounds = (0,) + cuts + (n,)
            if all(sum(b[bounds[j]:bounds[j + 1]]) == sum(e[bounds[j]:bounds[j + 1]]) for j in range(m)): cnt += 1
        print(cnt % (10**9 + 7))
