"""gen.py SEED SIZE | --max: NOIP 2020 P4. n in 1..=5e5 steps (c_i in 1..=k, d_i = +-1), k in
1..=10 dimensions, w_i in 1..=1e9. Small sizes use small fields and routes that often return to
their start (the -1 case) or drift slowly; --max is n = 5e5, k = 10, w_i = 1e9 with a drift of
one per round in every dimension, so walks last about 1e9 rounds."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500_000 if MAX else r.randint(1, min(500_000, 1 + 2 * S))
k = 10 if MAX else r.randint(1, min(10, 1 + S // 3))
w = [10**9] * k if MAX else [r.randint(1, r.choice([3, 1 + S, 10**9])) for _ in range(k)]
steps = []
if MAX or r.random() < 0.5:
    # balanced walk, then (for MAX: one extra +1 per dimension) so drifts are small
    half = (n - (k if MAX else 0)) // 2
    for _ in range(half):
        steps += [(c := r.randint(1, k), 1), (c, -1)]
    r.shuffle(steps)
    if MAX:
        steps += [(j, 1) for j in range(1, k + 1)]
    while len(steps) < n:
        steps.append((r.randint(1, k), r.choice([-1, 1])))
    steps = steps[:n]
else:
    steps = [(r.randint(1, k), r.choice([-1, 1])) for _ in range(n)]
print(n, k)
print(*w)
for c, d in steps:
    print(c, d)
