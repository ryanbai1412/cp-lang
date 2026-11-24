#!/usr/bin/env python3
# Plants a permutation q and prints the edge union it produces, so a valid q exists.
import random, sys
MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 0 if MAX else int(sys.argv[2])
# --max: sum n^2 = 9e6 as one n = 3000 test planted from a random q, so every row scan is
# 3000 long and S is dense
T = 1 if MAX else rng.randint(1, 5)
print(T)
for _ in range(T):
    n = 3000 if MAX else rng.randint(2, max(3, size * 2))
    q = rng.sample(range(1, n + 1), n)
    grid = [["0"] * n for _ in range(n)]
    p = q[:]
    for i in sorted(range(n), key=lambda i: q[i]):
        stack, parent = [], [-1] * n
        for j, v in enumerate(p):
            last = -1
            while stack and p[stack[-1]] < v:
                last = stack.pop()
            if last >= 0:
                parent[last] = j
            if stack:
                parent[j] = stack[-1]
            stack.append(j)
        for j, par in enumerate(parent):
            if par >= 0:
                grid[j][par] = "1"
        p[i] += n
    print(n)
    for row in grid:
        print("".join(row))
