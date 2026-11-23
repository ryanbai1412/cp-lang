import sys
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n, m = int(next(it)), int(next(it))
    adj = [[] for _ in range(n)]
    for _ in range(m):
        u, v = int(next(it)) - 1, int(next(it)) - 1; adj[u].append(v); adj[v].append(u)
    cnt = sum(all(any((c >> u & 1) != (c >> v & 1) for u in adj[v]) for v in range(n)) for c in range(1 << n))
    print("Brain Melt" if cnt == 0 else cnt if cnt <= 2 else "Many")
