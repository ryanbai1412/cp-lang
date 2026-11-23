#!/usr/bin/env python3
# checker.py input output [answer]: NO exactly when the reference says NO; otherwise at most 2023
# streets u != v in 0..n with 0 <= b <= W whose widest-car and widest-bike bottlenecks are C and B.
import sys
t = list(map(int, open(sys.argv[1]).read().split()))
n, W = t[0], t[1]; pos = 2
C = [[0] * n for _ in range(n)]; B = [[0] * n for _ in range(n)]
for M in (C, B):
    for j in range(1, n):
        for i in range(j): M[i][j] = M[j][i] = t[pos]; pos += 1
out = open(sys.argv[2]).read().split()
want_no = len(sys.argv) > 3 and open(sys.argv[3]).read().split()[:1] == ["NO"]
if out == ["NO"]:
    if len(sys.argv) > 3 and not want_no: sys.exit("NO, but the reference builds a network")
    print("ok"); sys.exit()
if want_no: sys.exit("a network, but the reference says NO")
out = list(map(int, out)); m = out[0]
if len(out) != 1 + 3 * m: sys.exit("wrong token count")
if m > 2023: sys.exit(f"{m} streets, at most 2023")
streets = [tuple(out[1 + 3 * i:4 + 3 * i]) for i in range(m)]
for u, v, b in streets:
    if not (0 <= u < n and 0 <= v < n and u != v and 0 <= b <= W): sys.exit(f"bad street {u} {v} {b}")
def bottleneck(key):
    comp = [[v] for v in range(n)]; at = list(range(n)); g = [[None] * n for _ in range(n)]
    for s in sorted(streets, key=key, reverse=True):
        a, c = at[s[0]], at[s[1]]
        if a == c: continue
        if len(comp[a]) < len(comp[c]): a, c = c, a
        for x in comp[a]:
            for y in comp[c]: g[x][y] = g[y][x] = key(s)
        for y in comp[c]: at[y] = a
        comp[a] += comp[c]; comp[c] = []
    return g
for name, want, got in (("car", C, bottleneck(lambda s: W - s[2])), ("bike", B, bottleneck(lambda s: s[2]))):
    for i in range(n):
        for j in range(i + 1, n):
            if got[i][j] != want[i][j]: sys.exit(f"{name} {i} {j}: {got[i][j]}, want {want[i][j]}")
print("ok")
