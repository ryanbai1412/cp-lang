import sys
from itertools import product
def ok(frogs, n):
    rows, cols = [0, 0], [0] * n
    for r, c in frogs: rows[r] += 1; cols[c] += 1
    moves = [[(r + dr, c + dc) for dr, dc in ((1, 0), (-1, 0), (0, 1), (0, -1)) if 0 <= r + dr < 2 and 0 <= c + dc < n] for r, c in frogs]
    for choice in product(*moves):
        r2, c2 = [0, 0], [0] * n
        for r, c in choice: r2[r] += 1; c2[c] += 1
        if r2 == rows and c2 == cols: return True
    return False
it = iter(sys.stdin.read().split()); t = int(next(it))
for _ in range(t):
    n = int(next(it)); g = [next(it), next(it)]
    cells = [(r, c) for r in range(2) for c in range(n) if g[r][c] == "f"]
    best = len(cells)
    for mask in range(1 << len(cells)):
        kept = [cells[i] for i in range(len(cells)) if mask >> i & 1]
        if len(cells) - len(kept) < best and ok(kept, n): best = len(cells) - len(kept)
    print(best)
