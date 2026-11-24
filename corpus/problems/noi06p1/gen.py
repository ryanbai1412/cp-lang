"""noi06p1 generator: N <= 10, 2^N plans, change costs 0..=500000 and the flow table F <= 500."""
import random
import sys


def emit(rng, n, fmax):
    m = 1 << n
    print(n)
    print(*(rng.randint(0, 1) for _ in range(m)))
    print(*(rng.randint(0, 500_000) for _ in range(m)))
    for i in range(1, m):
        print(*(rng.randint(0, fmax) for _ in range(m - i)))


if sys.argv[1] == "--max":
    emit(random.Random(0), 10, 500)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
emit(rng, rng.randint(1, max(1, min(10, size.bit_length()))), rng.choice([1, 10, 500]))
