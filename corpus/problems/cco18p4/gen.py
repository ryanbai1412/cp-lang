#!/usr/bin/env python3
"""Hidden test for CCO 2018 P4: `gen.py SEED [MAXSIDE] | --max` prints `R C K`, `N`, then N tokens.

tests/NN.case are hidden tests; NN.in / NN.ans are transcripts of spec.cp against them:
`interactor.py tests/NN.case --log-in tests/NN.in --log-out tests/NN.ans -- cpl run spec.cp`."""
import random, sys

MAX = sys.argv[1] == "--max"  # a 1e7 x 1e7 field, K = 75, 100 scattered points
g = random.Random(0 if MAX else int(sys.argv[1]))
side = int(sys.argv[2]) if len(sys.argv) > 2 and not MAX else 10**7
sub = 4 if MAX else g.randrange(5)
if sub == 0:
    r, c, k = 1, g.randint(1, min(90, side)), 90
elif sub == 1:
    r, c, k = 1, g.randint(1, side), 90
else:
    r, c, k = g.randint(1, side), g.randint(1, side), [170, 100, 75][sub - 2]
if MAX:
    r, c = side, side
elif g.random() < 0.3:
    r, c = (r, side) if r == 1 else (side, side)
n = 100 if MAX else g.randint(1, 100) if g.random() < 0.7 else g.randint(1, 4)
cluster = not MAX and g.random() < 0.3
pts = []
for _ in range(n):
    if cluster and pts:
        x, y = g.choice(pts)
        pts.append((min(r, max(1, x + g.randint(-2, 2))), min(c, max(1, y + g.randint(-2, 2)))))
    else:
        pts.append((g.randint(1, r), g.randint(1, c)))
print(r, c, k)
print(n)
for x, y in pts:
    print(x, y)
