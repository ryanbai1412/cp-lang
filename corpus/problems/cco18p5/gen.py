"""gen.py SEED SIZE | --max: CCO 2018 P5. N <= 1e6 qualities <= 1e9, window K <= N, Q <= 1e5
point assignments (small cases use few distinct values, so ties are common). --max is N = 1e6,
Q = 1e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 6 if MAX else r.randint(2, 2 + 2 * S)
k = r.randint(2, 1000) if MAX else r.randint(2, n)
q = 10 ** 5 if MAX else r.randint(0, 1 + S)
hi = 10 ** 9 if MAX or r.random() < .3 else r.randint(1, 6)
out = [f"{n} {k} {q}"] + [str(r.randint(1, hi)) for _ in range(n)]
out += [f"{r.randint(1, n)} {r.randint(1, hi)}" for _ in range(q)]
sys.stdout.write("\n".join(out) + "\n")
