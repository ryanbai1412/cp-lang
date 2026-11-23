import sys
from itertools import combinations
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, k = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    seen = [set() for _ in range(k)]
    for c in combinations(a, k):
        for i, v in enumerate(c): seen[i].add(v)
    print(sum(len(s) == 1 for s in seen))
