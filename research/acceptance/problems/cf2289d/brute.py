import sys
from collections import deque
def solve(a):
    n = len(a); dist = {a: 0}; q = deque([a])
    while q:
        b = q.popleft()
        if min(b) >= 0: return dist[b]
        for i in range(1, n):
            c = list(b); v = b[i]; c[i - 1] += v; c[i] = -v
            if i + 1 < n: c[i + 1] += v
            c = tuple(c)
            if c not in dist: dist[c] = dist[b] + 1; q.append(c)
    return -1
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); print(solve(tuple(int(next(it)) for _ in range(n))))
