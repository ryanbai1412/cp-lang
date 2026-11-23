import sys
from collections import deque
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, m = int(next(it)), int(next(it))
    w, x, y, z = (int(next(it)) - 1 for _ in range(4))
    adj = {(a, b): [] for a in range(n) for b in range(n)}
    for _ in range(m):
        a, b = int(next(it)) - 1, int(next(it)) - 1
        for c in range(n):
            adj[(c, a)].append((c, b)); adj[(c, b)].append((c, a))
        adj[(a, b)].append((b, a)); adj[(b, a)].append((a, b))
    dist = {(w, x): 0}; q = deque([(w, x)])
    while q:
        u = q.popleft()
        for v in adj[u]:
            if v not in dist: dist[v] = dist[u] + 1; q.append(v)
    print(dist[(y, z)])
