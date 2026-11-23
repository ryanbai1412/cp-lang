"""ioi09p6 generator: `N S` and an N x N map of T/G/M/D/H with one M, one D, at least one H,
a G path from M to D and from some hive to M (the statement's guarantees)."""
import random
import sys
from collections import deque

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 12
rng = random.Random(seed)


def reach(g, n, src, ok):
    seen = set(src)
    q = deque(src)
    while q:
        r, c = q.popleft()
        for nr, nc in ((r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)):
            if 0 <= nr < n and 0 <= nc < n and (nr, nc) not in seen and ok(g[nr][nc]):
                seen.add((nr, nc))
                q.append((nr, nc))
    return seen


def valid(g, n):
    cells = {ch: [(r, c) for r in range(n) for c in range(n) if g[r][c] == ch] for ch in "MDH"}
    if len(cells["M"]) != 1 or len(cells["D"]) != 1 or not cells["H"]:
        return False
    m, d = cells["M"][0], cells["D"][0]
    from_m = reach(g, n, [m], lambda ch: ch == "G")
    near = lambda p: any(abs(p[0] - x[0]) + abs(p[1] - x[1]) <= 1 for x in from_m)
    return near(d) and any(near(h) for h in cells["H"])


def snake(n):
    # A winding corridor: rows of grass joined at alternating ends, so walks are long.
    g = [["T"] * n for _ in range(n)]
    for r in range(0, n, 2):
        for c in range(n):
            g[r][c] = "G"
        if r + 1 < n:
            g[r + 1][n - 1 if r // 2 % 2 == 0 else 0] = "G"
    return g


while True:
    n = rng.randint(2, max(2, size))
    s = rng.choice([1, 1, 2, 3, rng.randint(1, 10), rng.randint(1, 1000), 1000])
    shape = rng.choice(["open", "trees", "snake"])
    if shape == "snake" and n >= 3:
        g = snake(n)
    else:
        dens = 0.0 if shape == "open" else rng.uniform(0.1, 0.4)
        g = [["T" if rng.random() < dens else "G" for _ in range(n)] for _ in range(n)]
    free = [(r, c) for r in range(n) for c in range(n) if g[r][c] == "G"]
    if len(free) < 3:
        continue
    picks = rng.sample(free, min(len(free), 2 + rng.choice([1, 1, 2, 3, rng.randint(1, 8)])))
    (mr, mc), (dr, dc) = picks[0], picks[1]
    g[mr][mc], g[dr][dc] = "M", "D"
    for r, c in picks[2:]:
        g[r][c] = "H"
    if valid(g, n):
        break
print(n, s)
for row in g:
    print("".join(row))
