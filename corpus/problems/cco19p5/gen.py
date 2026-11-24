"""gen.py SEED SIZE | --max: CCO 2019 P5. N, M <= 1e5 and M distinct pairs a < b (small cases are
dense so closures cascade). --max is N = M = 1e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 5 if MAX else r.randint(2, 3 + 3 * S)
m = 10 ** 5 if MAX else r.randint(1, min(n * (n - 1) // 2, 1 + 4 * S))
pairs = set()
while len(pairs) < m:
    a, b = sorted(r.sample(range(1, n + 1), 2))
    pairs.add((a, b))
out = [f"{n} {m}"] + [f"{a} {b}" for a, b in sorted(pairs, key=lambda _: r.random())]
sys.stdout.write("\n".join(out) + "\n")
