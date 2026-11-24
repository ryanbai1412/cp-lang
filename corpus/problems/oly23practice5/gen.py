"""oly23practice5 generator: T cases, each n <= 3e5 values a_i in 1..=n. The statement is not
public; the bounds are the port's and the accepted C++'s (which asserts 1 <= a_i <= n), and
--max is one case at n = 3e5."""
import random
import sys


def case(rng, n, kinds):
    """Runs of alternating pairs drawn from `kinds` values, with noise between them."""
    vals = list(range(1, min(n, kinds) + 1))
    a = []
    while len(a) < n:
        x, y = rng.choice(vals), rng.choice(vals)
        a += [x, y] * rng.randint(1, 6)
        if rng.random() < 0.3:
            a.append(rng.randint(1, n))
    return a[:n]


if sys.argv[1] == "--max":
    rng = random.Random(0)
    n = 300000
    print(1)
    print(n)
    # many short alternating runs over ~800 values: ~3e5 distinct penalised pairs
    print(*case(rng, n, 800))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, max(1, min(300000, size)))
    print(n)
    print(*case(rng, n, rng.choice([2, 3, n])))
