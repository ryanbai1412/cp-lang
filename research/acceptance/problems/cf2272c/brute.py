import sys
from itertools import combinations
def fine(b, k): return all(any(c != x and abs(c - x) <= k for c in b) for x in b)
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, x = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]
    k = 1
    while True:
        free = [c for c in range(0, a[-1] + k + 1) if c not in a]
        if any(fine(set(a) | set(y), k) for s in range(x + 1) for y in combinations(free, s)): break
        k += 1
    print(k)
