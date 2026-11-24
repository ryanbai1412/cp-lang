"""noi20p1 generator: n <= 50 cities each with an out-road, n <= m <= 501 roads of 1..=5 days,
c_i <= 52501, T <= 1e9, and k <= 200 festivals on distinct days with bonus <= 1e9."""
import random
import sys


def emit(rng, n, m, t, k):
    print(n, m, t, k)
    print(*(rng.randint(1, 52501) for _ in range(n)))
    roads = [(u, rng.choice([v for v in range(1, n + 1) if v != u])) for u in range(1, n + 1)]
    roads += [tuple(rng.sample(range(1, n + 1), 2)) for _ in range(m - n)]
    rng.shuffle(roads)
    for u, v in roads:
        print(u, v, rng.randint(1, 5))
    for d in sorted(rng.sample(range(1, t + 1), k)):
        print(d, rng.randint(1, n), rng.randint(1, 10**9))


if sys.argv[1] == "--max":
    emit(random.Random(0), 50, 501, 10**9, 200)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(2, max(2, min(50, size)))
t = rng.randint(1, rng.choice([20, 10**9]))
emit(rng, n, rng.randint(n, min(501, max(n, 3 * n))), t, rng.randint(0, min(200, t, size)))
