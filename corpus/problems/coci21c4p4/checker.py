#!/usr/bin/env python3
# checker.py input output [answer]: the reference's least maximum distance, then exactly k
# distinct towns in 1..=n whose nearest park is at most that far from every town.
import heapq, sys
t = list(map(int, open(sys.argv[1]).read().split()))
n, k = t[0], t[1]
adj = [[] for _ in range(n + 1)]
for i in range(n - 1):
    a, b, w = t[2 + 3 * i: 5 + 3 * i]
    adj[a].append((b, w)); adj[b].append((a, w))
out = list(map(int, open(sys.argv[2]).read().split()))
if len(out) != 1 + k: sys.exit(f"{len(out) - 1} towns for k = {k}")
r = out[0]
if len(sys.argv) > 3:
    best = int(open(sys.argv[3]).read().split()[0])
    if r != best: sys.exit(f"distance {r}, least {best}")
parks = out[1:]
if len(set(parks)) != k or not all(1 <= p <= n for p in parks): sys.exit("towns out of range or repeated")
dist = [None] * (n + 1)
pq = [(0, p) for p in parks]
while pq:
    d, u = heapq.heappop(pq)
    if dist[u] is not None: continue
    dist[u] = d
    for v, w in adj[u]:
        if dist[v] is None: heapq.heappush(pq, (d + w, v))
far = max(dist[1:])
if far != r: sys.exit(f"farthest town is {far} from a park, printed {r}")
print("ok")
