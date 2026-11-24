#!/usr/bin/env python3
"""gen.py seed size | --max: CCO 2017 Vera and Modern Art. N, Q <= 2e5, strokes "x y v" with
x, y <= 1e18, v <= 1e4, queries "r c" with r, c <= 1e18. --max: N = Q = 2e5 with x and y of
every bit length up to 60 (deep tries), most queries landing on a stroke's lattice."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else max(1, int(sys.argv[2]))
n = 200_000 if MAX else r.randint(1, S)
q = 200_000 if MAX else r.randint(1, S)
TOP = 10 ** 18 if MAX else 10 ** r.randint(1, 18)
def coord():
    return min(TOP, r.randint(1, 1 << r.randint(1, TOP.bit_length())))
strokes = [(coord(), coord(), r.randint(1, 10_000)) for _ in range(n)]
out = [f"{n} {q}"] + [f"{x} {y} {v}" for x, y, v in strokes]
for _ in range(q):
    if r.random() < 0.8:
        x, y, _ = r.choice(strokes)
        a, b = 1 << (x.bit_length() - 1), 1 << (y.bit_length() - 1)
        rr, c = x + a * r.randint(0, (TOP - x) // a), y + b * r.randint(0, (TOP - y) // b)
    else:
        rr, c = coord(), coord()
    out.append(f"{rr} {c}")
sys.stdout.write("\n".join(out) + "\n")
