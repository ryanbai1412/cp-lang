"""gen.py SEED SIZE | --max: Codeforces 1083E (The Fair Nut and Rectangles). n <= 1e6 rectangles
(0,0)-(x,y), 1 <= x, y <= 1e9, none nested, 0 <= a <= x*y. Small cases use a small grid.
--max is n = 1e6 with x rising and y falling across the full range, a near x*y/2."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(1, 1 + S)
hi = 10**9 if MAX or r.random() < 0.3 else max(n, 3 * S)
xs = sorted(r.sample(range(1, hi + 1), n))
ys = sorted(r.sample(range(1, hi + 1), n), reverse=True)
rows = []
for x, y in zip(xs, ys):
    a = r.randint(x * y // 4, x * y * 3 // 4) if MAX else r.randint(0, x * y)
    rows.append(f"{x} {y} {a}\n")
r.shuffle(rows)
sys.stdout.write(f"{n}\n" + "".join(rows))
