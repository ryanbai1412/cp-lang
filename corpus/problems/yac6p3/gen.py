#!/usr/bin/env python3
"""Hidden case for YAC 6 P3: `gen.py SEED [MAXSIDE]` prints `N M`, then `A B C D` (distinct cells).

tests/NN.case are hidden cases; NN.in / NN.ans are transcripts of spec.cp against them:
`interactor.py tests/NN.case --log-in tests/NN.in --log-out tests/NN.ans -- cpl run spec.cp`."""
import random, sys

g = random.Random(int(sys.argv[1]))
side = max(2, int(sys.argv[2])) if len(sys.argv) > 2 else 10**9
pick = lambda: g.randint(1, side) if g.random() < 0.7 else g.randint(1, min(side, 6))
while True:
    n, m = pick(), pick()
    if g.random() < 0.25:
        m = 1  # subtask 1
    if g.random() < 0.15:
        n, m = (side, side) if g.random() < 0.5 else (n, side)
    if n * m >= 2:
        break


def cell():
    s = g.random()
    if s < 0.2:  # a corner
        return g.choice([1, n]), g.choice([1, m])
    if s < 0.35:  # on an edge
        return (g.choice([1, n]), g.randint(1, m)) if g.random() < 0.5 else (g.randint(1, n), g.choice([1, m]))
    return g.randint(1, n), g.randint(1, m)


while True:
    p = cell()
    s = g.random()
    if s < 0.2:  # a neighbour of p, often the tie-breaking shape
        dx, dy = g.choice([(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)])
        q = (p[0] + dx, p[1] + dy)
    elif s < 0.35:  # the same row or column
        q = (p[0], g.randint(1, m)) if g.random() < 0.5 else (g.randint(1, n), p[1])
    else:
        q = cell()
    if 1 <= q[0] <= n and 1 <= q[1] <= m and q != p:
        break
if g.random() < 0.5:
    p, q = q, p
print(n, m)
print(*p, *q)
