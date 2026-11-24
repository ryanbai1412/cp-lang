"""gen.py SEED SIZE | --max: CCC 2014 S4. n <= 1000 panes with corners in 0..=K and tints, a
threshold T. Small cases use a coarse grid so edges and corners coincide. --max is n = 1000,
K = 1e9, tints up to 1e6."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, 1 + 3 * S)
K = 10 ** 9 if MAX else r.choice([4, 10, 10 ** 9])
tmax = 10 ** 6 if MAX else r.choice([3, 10 ** 6])
print(n); print(r.randint(1, tmax * (1 + n // 3)))
for _ in range(n):
    x0, x1 = sorted(r.sample(range(K + 1), 2)) if K < 100 else sorted([r.randint(0, K - 1), r.randint(1, K)])
    y0, y1 = sorted(r.sample(range(K + 1), 2)) if K < 100 else sorted([r.randint(0, K - 1), r.randint(1, K)])
    if x0 == x1: x1 += 1
    if y0 == y1: y1 += 1
    print(x0, y0, x1, y1, r.randint(1, tmax))
