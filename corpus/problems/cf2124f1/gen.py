#!/usr/bin/env python3
import sys, random

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = 100 and m = 5000: half of all cells banned, mostly away from
    # the main diagonals so many runs survive and the count stays large.
    rng = random.Random(2124)
    n, m = 100, 5000
    cells = [(i, x) for i in range(1, n + 1) for x in range(1, n + 1)]
    cells.sort(key=lambda c: (-min((c[0] - c[1]) % n, (c[1] - c[0]) % n) + rng.random() * 30))
    print(1)
    print(n, m)
    for i, x in sorted(cells[:m], key=lambda _: rng.random()):
        print(i, x)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(2, 10)
    all_pairs = [(i, x) for i in range(1, n + 1) for x in range(1, n + 1)]
    m = rng.randint(0, min(len(all_pairs), 12))
    chosen = rng.sample(all_pairs, m)
    print(n, m)
    for i, x in chosen:
        print(i, x)
