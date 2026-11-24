"""gen.py SEED SIZE | --max: CCO 2017 P5. n <= 2e5 people, each A_i in 0..=n and B_i in 0..=1e4.
--max is n = 2e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2 * 10 ** 5 if MAX else r.randint(1, 2 + 5 * S)
hi = 10 ** 4 if MAX or r.random() < .5 else 5
out = [str(n)] + [f"{r.randint(0, n)} {r.randint(0, hi)}" for _ in range(n)]
sys.stdout.write("\n".join(out) + "\n")
