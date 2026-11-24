"""ioi06p4 (The Valley of Mexico) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

c (3 <= c <= 1000), then n, then n distinct agreements "u v" (u != v, cities 1..=c).
Seeds hide a non-crossing route (a growing arc of the circle) among random agreements, or not.
--max: c = 1000 and every one of the c(c - 1)/2 pairs but one in 50 agrees.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    c = 1000
    pairs = [(u, v) for u in range(1, c + 1) for v in range(u + 1, c + 1) if rng.randrange(50)]
else:
    c = rng.randint(3, min(1000, max(3, int(sys.argv[2]))))
    edges = set()
    if rng.random() < 0.7:
        l = r = rng.randint(0, c - 1)
        at = l
        for _ in range(c - 1):
            if rng.random() < 0.5:
                l = (l - 1) % c
                nxt = l
            else:
                r = (r + 1) % c
                nxt = r
            edges.add((min(at, nxt) + 1, max(at, nxt) + 1))
            at = nxt
    for _ in range(rng.randint(0, c * (c - 1) // 4)):
        u, v = rng.sample(range(1, c + 1), 2)
        edges.add((min(u, v), max(u, v)))
    pairs = list(edges)
    rng.shuffle(pairs)
    pairs = [(v, u) if rng.random() < 0.5 else (u, v) for u, v in pairs]
print(c)
print(len(pairs))
print("\n".join(f"{u} {v}" for u, v in pairs))
