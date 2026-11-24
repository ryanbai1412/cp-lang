#!/usr/bin/env python3
"""gen.py seed size | --max: APIO 2007 Zoo. 10 <= n <= 1e4 pens in a circle, c <= 5e4 children
"E F L x1..xF y1..yL", each naming distinct pens among E..E+4 (wrapping), a pen not both feared
and loved. --max: n = 1e4, c = 5e4, children at every pen (also across the wrap) naming many
pens, so every 2^5 view state of each pen holds children."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10_000 if MAX else max(10, min(10_000, int(sys.argv[2])))
c = 50_000 if MAX else r.randint(1, max(1, min(50_000, 3 * n)))
out = [f"{n} {c}"]
for i in range(c):
    e = (i % n) + 1 if MAX and i < 4 * n else r.randint(1, n)
    view = [(e - 1 + d) % n + 1 for d in range(5)]
    picked = r.sample(view, r.randint(1, 5))
    split = r.randint(0, len(picked))
    fear, love = picked[:split], picked[split:]
    out.append(" ".join(map(str, [e, len(fear), len(love)] + fear + love)))
sys.stdout.write("\n".join(out) + "\n")
