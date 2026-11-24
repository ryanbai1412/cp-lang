"""NOI 2016 P2 (Grid): T <= 20 tests, each `n m c` (1 <= n, m <= 1e9, 0 <= c <= min(n m, 1e5),
sum c <= 1e5) and c distinct cricket cells `x y`.

`gen.py SEED SIZE`: small grids (often 1 or 2 wide) about SIZE cells, any number of crickets.
`gen.py --max`: T = 20, sum c = 1e5: one 1e9 x 1e9 test with 81000 scattered crickets (every
cricket's neighbourhood graph is built, answer 2), then 19 tests of 1000 crickets: clumps,
walls that nearly cut the grid, 1-wide and 2-wide strips and grids with few fleas left.
"""
import random
import sys


def test(g, n, m, cells):
    cells = list(cells)
    g.shuffle(cells)
    return [f"{n} {m} {len(cells)}"] + [f"{x} {y}" for x, y in cells]


def scattered(g, n, m, c):
    s = set()
    while len(s) < c:
        s.add((g.randint(1, n), g.randint(1, m)))
    return s


if sys.argv[1:] == ["--max"]:
    g = random.Random(1)
    big = 10**9
    out = ["20"] + test(g, big, big, scattered(g, big, big, 81000))
    for t in range(19):
        kind = t % 5
        if kind == 0:  # a dense clump with holes inside
            out += test(g, big, big, scattered(g, 40, 40, 1000))
        elif kind == 1:  # a wall across a 999-wide grid with one gap
            gap = g.randint(1, 999)
            wall = {(500, y) for y in range(1, 1000) if y != gap}
            out += test(g, big, 999, wall | scattered(g, 499, 999, 1000 - len(wall)) - {(500, gap)})
        elif kind == 2:  # a 1-wide strip
            out += test(g, 1, big, {(1, y) for y in g.sample(range(1, 10**6), 1000)})
        elif kind == 3:  # a 2-wide strip
            out += test(g, 2, big, scattered(g, 2, 3000, 1000))
        else:  # few fleas left
            n, m = 25, 40
            out += test(g, n, m, scattered(g, n, m, 1000 - (t % 3)))
    print("\n".join(out))
    sys.exit()

g = random.Random(int(sys.argv[1]))
size = max(1, int(sys.argv[2]))
t = g.randint(1, 5)
out = [str(t)]
for _ in range(t):
    n = g.choice([1, 2, g.randint(1, max(1, int(size**0.5))), g.randint(1, size)])
    m = g.randint(1, max(1, size // n))
    if g.random() < 0.2:
        n, m = g.choice([(n * 10**5, m), (10**9, 10**9)])
    cap = min(n * m, size)
    c = g.choice([0, g.randint(0, cap), cap, max(0, n * m - g.randint(0, 3)) if n * m <= size else cap])
    out += test(g, n, m, scattered(g, n, m, c) if n * m > 3 * c else set(g.sample([(x, y) for x in range(1, n + 1) for y in range(1, m + 1)], c)))
print("\n".join(out))
