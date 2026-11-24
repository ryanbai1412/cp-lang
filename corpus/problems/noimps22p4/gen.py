"""noimps22p4 generator: n <= 1e6 values in 1..=2000, then m <= 1500 queries of distinct
primes <= 2000 with total length <= 18000."""
import random
import sys

PRIMES = [p for p in range(2, 2001) if all(p % d for d in range(2, int(p**0.5) + 1))]


def emit(rng, n, hi, lens):
    print(n)
    print(*(rng.randint(1, hi) for _ in range(n)))
    print(len(lens))
    for c in lens:
        print(c, *rng.sample(PRIMES, c))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    print(10**6)
    print(*(rng.randint(1, 2000) for _ in range(10**6)))
    print(1500)
    # Inclusion-exclusion over the queried primes <= 41 dominates: query many of them.
    for _ in range(1500):
        s = rng.randint(10, 12)
        print(12, *rng.sample(PRIMES[:13], s), *rng.sample(PRIMES[13:], 12 - s))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
m = rng.randint(1, max(1, min(1500, size)))
emit(rng, rng.randint(1, max(1, min(10**6, 10 * size))), rng.choice([30, 2000]),
     [rng.randint(1, min(len(PRIMES), 18000 // m, max(1, size))) for _ in range(m)])
