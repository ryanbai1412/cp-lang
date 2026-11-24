#!/usr/bin/env python3
"""gen.py seed size | --max: BOI 2017 Friends. n <= 2500, p + q <= 15, friend lists with total
length <= 30000. --max: n = 2500, p = 8, q = 7 and a symmetric graph of 15000 friendships with
every degree at most p + q - 1 = 14, so each search runs its full 2^(p+q) tree."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 2500 if MAX else r.randint(1, max(1, int(sys.argv[2])))
p, q = (8, 7) if MAX else (lambda p: (p, r.randint(0, 15 - p)))(r.randint(1, 15))
cap = p + q - 1 if MAX or r.random() < 0.8 else p + q
want = 15_000 if MAX else r.randint(0, n * 3)
adj = [set() for _ in range(n)]
edges = 0
for _ in range(want * 4):
    if edges >= want: break
    u, v = r.randrange(n), r.randrange(n)
    if MAX: v = (u + r.randint(1, 40)) % n
    if u == v or v in adj[u] or len(adj[u]) >= cap or len(adj[v]) >= cap: continue
    adj[u].add(v); adj[v].add(u); edges += 1
if not MAX and n > 1 and r.random() < 0.15:
    u = r.randrange(n); v = (u + 1) % n
    adj[u].add(v)  # one-sided claim: detention
print(n, p, q)
for u in range(n):
    print(len(adj[u]), *sorted(adj[u]))
