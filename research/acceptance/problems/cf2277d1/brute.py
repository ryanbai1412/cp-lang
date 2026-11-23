import sys
from collections import deque
MOD, BFS_MAX, MARGIN = 998244353, 6000, 80
def bfs(r):
    # shortest paths over (x, k): x the value, k the current run of Operation 1. Every prefix of a
    # path to n in f(n) <= MARGIN steps stays in [-MARGIN, n + MARGIN], so the box is exact.
    lo, hi = -MARGIN, r + MARGIN
    dist = {(0, 0): 0}; q = deque([(0, 0)]); best = {}
    while q:
        x, k = q.popleft(); d = dist[(x, k)]
        if 0 <= x <= r and x not in best: best[x] = d
        for s in ((x + (1 << k), k + 1), (x - 1, 0)):
            if lo <= s[0] <= hi and s not in dist: dist[s] = d + 1; q.append(s)
    assert all(best[n] <= MARGIN for n in range(r + 1))
    return best
def window(l, r):
    # samples past BFS reach: runs as a set of distinct lengths a >= 2, i.e. a mask V with bits 0, 1
    # clear; f(n) = min V + sum(a - 1) - n over V - 2 popcount(V) >= n - 1, scanned exhaustively
    # over every V up to r + 2000 (V = ceil4(n + 123) is always feasible with f < 1900).
    f = {}
    vs = []
    for v in range(max(4, l - l % 4), r + 2001, 4):
        bits = [a for a in range(2, v.bit_length()) if v >> a & 1]
        vs.append((v - 2 * len(bits), v + sum(a - 1 for a in bits)))
    vs.sort(reverse=True); i = 0; cur = None
    for n in range(r, l - 1, -1):
        if n <= 1: f[n] = n; continue
        while i < len(vs) and vs[i][0] >= n - 1:
            cur = vs[i][1] if cur is None else min(cur, vs[i][1]); i += 1
        f[n] = cur - n
    return f
it = iter(sys.stdin.read().split()); t = int(next(it))
qs = [(int(next(it)), int(next(it))) for _ in range(t)]
small = max([r for l, r in qs if r <= BFS_MAX], default=-1)
best = bfs(small) if small >= 0 else {}
for l, r in qs:
    f = best if r <= BFS_MAX else window(l, r)
    print(sum(f[n] for n in range(l, r + 1)) % MOD)
