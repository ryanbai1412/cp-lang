"""gen.py SEED SIZE | --max: nccc10s5 mall. An R x C grid (4 <= R * C <= 99) of . # U D with
one U and one D, every non-pillar cell connected to the rest. SIZE caps R * C; cells cut off
from the escalators' component become pillars. --max is 11 x 9 with sparse pillars, so the
profile DP over the 9-wide side carries the most components."""
import random, sys


def connected(g, R, C):
    cells = [(i, j) for i in range(R) for j in range(C) if g[i][j] != "#"]
    seen, stack = {cells[0]}, [cells[0]]
    while stack:
        x, y = stack.pop()
        for a, b in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if 0 <= a < R and 0 <= b < C and g[a][b] != "#" and (a, b) not in seen:
                seen.add((a, b))
                stack.append((a, b))
    return seen


def grid(r, R, C, density):
    while True:
        g = [["#" if r.random() < density else "." for _ in range(C)] for _ in range(R)]
        free = [(i, j) for i in range(R) for j in range(C) if g[i][j] == "."]
        if len(free) < 2:
            continue
        seen = connected(g, R, C)
        if len(seen) < 2:
            continue
        for i in range(R):
            for j in range(C):
                if (i, j) not in seen:
                    g[i][j] = "#"
        (ux, uy), (dx, dy) = r.sample(sorted(seen), 2)
        g[ux][uy], g[dx][dy] = "U", "D"
        return g


if sys.argv[1] == "--max":
    r, R, C, density = random.Random(2010), 11, 9, 0.12
else:
    r = random.Random(int(sys.argv[1]))
    cap = max(4, min(99, 4 * int(sys.argv[2])))
    while True:
        R, C = r.randint(1, cap), r.randint(1, cap)
        if 4 <= R * C <= cap:
            break
    density = r.choice([0.0, 0.1, 0.25, 0.4])
g = grid(r, R, C, density)
print(R, C)
print("\n".join("".join(row) for row in g))
