#!/usr/bin/env python3
# checker.py input output [answer]: n^2 grids with at most k zeros, then a table a_0..a_{n^2}
# under which the memoryless robot (2n - 2 moves, each by a_S, S = ones inside) ends on
# exactly the target cell of every grid.
import sys
n = int(open(sys.argv[1]).read().split()[0])
k = {3: 9, 20: 24}.get(n, n + 2)
tok = open(sys.argv[2]).read().split()
if len(tok) != n * n * n + 1: sys.exit(f"want {n * n * n + 1} tokens, got {len(tok)}")
table = tok[-1]
if len(table) != n * n + 1 or set(table) - set("UDLR"): sys.exit(f"bad table {table}")
for t in range(n * n):
    g = tok[t * n:(t + 1) * n]
    if any(len(row) != n or set(row) - set("01") for row in g): sys.exit(f"grid {t + 1}: bad rows")
    if sum(row.count("0") for row in g) > k: sys.exit(f"grid {t + 1}: more than {k} zeros")
    pre = [[0] * (n + 1) for _ in range(n + 1)]
    for i in range(n):
        for j in range(n):
            pre[i + 1][j + 1] = pre[i][j + 1] + pre[i + 1][j] - pre[i][j] + (g[i][j] == "1")
    x0, y0, x1, y1 = 0, 0, n - 1, n - 1
    for _ in range(2 * n - 2):
        s = pre[x1 + 1][y1 + 1] - pre[x0][y1 + 1] - pre[x1 + 1][y0] + pre[x0][y0]
        a = table[s]
        if a == "U": x0 += 1
        elif a == "D": x1 -= 1
        elif a == "L": y0 += 1
        else: y1 -= 1
        if x0 > x1 or y0 > y1: sys.exit(f"grid {t + 1}: empty rectangle")
    if (x0, x1, y0, y1) != (t // n, t // n, t % n, t % n):
        sys.exit(f"grid {t + 1}: ends on ({x0 + 1}, {y0 + 1}) not target")
print("ok")
