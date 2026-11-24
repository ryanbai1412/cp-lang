"""ioi99p5 (Flatten) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N (2 <= N <= 200), then N piles (0 <= C <= 2000) that can be flattened: flat piles of a random
height, then reversed moves (a pile takes m chips from each neighbour). --max: N = 200 and 2000
reversed moves around a high average.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 200 if MAX else max(2, int(sys.argv[2]))
n = size if MAX else rng.randint(2, min(200, size))
c = [rng.randint(500, 1500) if MAX else rng.randint(0, 2000)] * n
for _ in range(2000 if MAX else rng.randint(1, 5 * size)):
    p = rng.randrange(n)
    nb = [q for q in (p - 1, p + 1) if 0 <= q < n]
    room = min([c[q] for q in nb] + [(2000 - c[p]) // len(nb)])
    if room > 0:
        m = rng.randint(1, room)
        c[p] += m * len(nb)
        for q in nb:
            c[q] -= m
print(n)
print(*c)
