#!/usr/bin/env python3
"""CF 2201F2: 2 <= n, q <= min(n^2, 2e6) distinct cells; sum n, sum q <= 2e6."""
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])


def rand_cells(n, q):
    seen = set()
    while len(seen) < q:
        seen.add((rng.randint(1, n), rng.randint(1, n)))
    cells = list(seen)
    rng.shuffle(cells)
    return cells


def staircase(n, q):
    # fill a Young diagram row by row from the top-left: monotone throughout.
    cells = []
    for r in range(1, n + 1):
        for c in range(1, n + 2 - r):
            if len(cells) == q:
                return cells
            cells.append((r, c))
    return cells


def full(n):
    cells = [(r, c) for r in range(1, n + 1) for c in range(1, n + 1)]
    rng.shuffle(cells)
    return cells


if size >= 1000:
    L = 2000000
    mode = int(sys.argv[1]) % 5
    tests = [[(L, rand_cells(L, L))], [(L, staircase(L, L))], [(1414, full(1414))],
             [(1999, staircase(1999, L))],
             [(2, full(2)) for _ in range(1000)]][mode]
else:
    tests = []
    for _ in range(rng.randint(1, 4)):
        n = rng.randint(2, 6)
        q = rng.randint(1, n * n)
        tests.append((n, rng.choice([rand_cells(n, q), staircase(n, q), full(n)[:q]])))
lines = [str(len(tests))]
for n, cells in tests:
    lines.append(f"{n} {len(cells)}")
    lines += [f"{r} {c}" for r, c in cells]
print("\n".join(lines))
