"""gen.py SEED SIZE | --max: CCO 2020 P1. N <= 1e5 friends at x in L..=R, v, h in 1..=100, Y <= 1e6,
|L|, |R| <= 1e9. --max is N = 1e5 over the full range."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 5 if MAX else r.randint(1, 2 + 3 * S)
w = 10 ** 9 if MAX or r.random() < .3 else r.randint(0, 30)
lo = -w if MAX else r.randint(-w, w)
hi = w if MAX else r.randint(lo, w)
y = 10 ** 6 if MAX or r.random() < .3 else r.randint(1, 5)
out = [f"{n} {lo} {hi} {y}"] + [f"{r.randint(lo, hi)} {r.randint(1, 100)} {r.randint(1, 100)}" for _ in range(n)]
sys.stdout.write("\n".join(out) + "\n")
