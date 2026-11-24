#!/usr/bin/env python3
"""CF 2201F2: 2 <= n, q <= min(n^2, 2e6) distinct cells; sum n, sum q <= 2e6."""
import random, sys
if sys.argv[1] == "--max":
    # one test at n = q = 2000000: a randomly grown Young diagram under random row and column
    # permutations, so the matrix stays monotone and every answer is YES
    def young(n, q, rng):
        """q cells of a randomly grown Young diagram, rows and columns permuted: monotone at every step."""
        row = [0] * (n + 2)
        row[0] = n
        addable, at = [1], {1: 0}
    
        def drop(r):
            i = at.pop(r)
            last = addable.pop()
            if last != r:
                addable[i] = last
                at[last] = i
    
        cells = []
        for _ in range(q):
            r = addable[rng.randrange(len(addable))]
            row[r] += 1
            cells.append((r, row[r]))
            if row[r] == row[r - 1] or row[r] == n:
                drop(r)
            if r < n and row[r + 1] == row[r] - 1 and r + 1 not in at:
                at[r + 1] = len(addable)
                addable.append(r + 1)
        pr, pc = list(range(1, n + 1)), list(range(1, n + 1))
        rng.shuffle(pr)
        rng.shuffle(pc)
        return [(pr[r - 1], pc[c - 1]) for r, c in cells]
    n = q = 2000000
    g = random.Random(1)
    out = ["1", f"{n} {q}"] + [f"{r} {c}" for r, c in young(n, q, g)]
    print("\n".join(out))
    sys.exit()

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
