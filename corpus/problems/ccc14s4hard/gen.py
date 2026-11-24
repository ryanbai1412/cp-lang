"""gen.py SEED SIZE | --max: CCC 2014 S4 (hard data). n panes with corners in 0..=1e9 (small
cases on a coarse grid so edges coincide), tints, a threshold. --max is n = 1e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 5 if MAX else r.randint(1, 1 + 4 * S)
K = 10 ** 9 if MAX else r.choice([4, 10, 10 ** 9])
tmax = 10 ** 6 if MAX else r.choice([3, 10 ** 6])
out = [str(n), str(r.randint(1, tmax * (1 + min(n, 300) // 3)))]
for _ in range(n):
    x0, x1 = sorted(r.sample(range(K + 1), 2))
    y0, y1 = sorted(r.sample(range(K + 1), 2))
    out.append(f"{x0} {y0} {x1} {y1} {r.randint(1, tmax)}")
sys.stdout.write("\n".join(out) + "\n")
