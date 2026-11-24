"""noi06p4 generator: N <= 5000 stations with costs P <= 100, M <= 50000 groups `A B C`, C <= 100."""
import random
import sys


def emit(rng, n, m, hi):
    print(n, m)
    print(*(rng.randint(0, hi) for _ in range(n)))
    for _ in range(m):
        a, b = rng.sample(range(1, n + 1), 2)
        print(a, b, rng.randint(0, hi))


if sys.argv[1] == "--max":
    emit(random.Random(0), 5000, 50000, 100)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(2, max(2, min(5000, size)))
emit(rng, n, rng.randint(1, max(1, min(50000, 3 * size))), rng.choice([5, 100]))
