"""ioi15p2 (Scales): `gen.py SEED SIZE`, or `gen.py --max`: hidden data for interactor.py.

T (1 <= T <= 18), then T lines each listing the coins 1..6 from lightest to heaviest.
--max: T = 18 distinct orders.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
t = 18 if MAX else rng.randint(1, min(18, max(1, int(sys.argv[2]))))
orders = set()
while len(orders) < t:
    o = list(range(1, 7))
    rng.shuffle(o)
    orders.add(tuple(o))
print(t)
for o in sorted(orders, key=lambda _: rng.random()):
    print(*o)
