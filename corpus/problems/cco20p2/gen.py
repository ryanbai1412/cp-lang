"""gen.py SEED SIZE | --max: CCO 2020 P2. n <= 2e5 deadlines in 1..=n, often feasible (each
deadline at least the rank of a random permutation's position, with slack). --max is n = 2e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2 * 10 ** 5 if MAX else r.randint(1, 2 + 4 * S)
if MAX or r.random() < .7:
    perm = list(range(1, n + 1)); r.shuffle(perm)
    d = [min(n, p + r.randint(0, 2)) for p in perm]
else:
    d = [r.randint(1, n) for _ in range(n)]
print(n); print(" ".join(map(str, d)))
