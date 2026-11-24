"""gen.py SEED SIZE | --max: CCO 2005 P6. A C x R screen (<= 1e4), n <= 5e4 windows given as
`left top right bottom` (borders included), m <= 5e4 clicks. Small cases use a small screen so
windows overlap and clicks land on borders. --max is n = m = 5e4 on a 1e4 x 1e4 screen."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
c = 10_000 if MAX else r.randint(1, 3 + S)
h = 10_000 if MAX else r.randint(1, 3 + S)
n = 50_000 if MAX else r.randint(1, 1 + 2 * S)
m = 50_000 if MAX else r.randint(0, 1 + 2 * S)
out = [f"{c} {h}", str(n)]
for _ in range(n):
    x1, x2 = sorted((r.randint(0, c), r.randint(0, c)))
    y1, y2 = sorted((r.randint(0, h), r.randint(0, h)))
    out.append(f"{x1} {y2} {x2} {y1}")
out += [str(m)] + [f"{r.randint(0, c)} {r.randint(0, h)}" for _ in range(m)]
sys.stdout.write("\n".join(out) + "\n")
