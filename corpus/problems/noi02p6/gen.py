"""noi02p6 generator: k distinct increasing primes 2 <= p < 1e4, each with 1 <= e <= 1e6."""
import random
import sys

PRIMES = [p for p in range(2, 10**4) if all(p % d for d in range(2, int(p**0.5) + 1))]
if sys.argv[1] == "--max":
    print(1000)
    for p in PRIMES[:1000]:
        print(p, 10**6)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
k = rng.randint(1, max(1, min(1000, size)))
ps = sorted(rng.sample(PRIMES[: max(k, min(len(PRIMES), 4 * size))], k))
print(k)
for p in ps:
    print(p, rng.choice([1, 2, rng.randint(1, 10), rng.randint(1, 10**6)]))
