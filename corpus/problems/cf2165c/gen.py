#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = 5e5 and q = 5e4: values mostly below 2^20 with rare high bits,
    # targets with high bits set, so most queries raise values bit after bit.
    rng = random.Random(2165)
    n, q = 500_000, 50_000
    print(1)
    print(n, q)
    print(*[rng.randrange(1 << 30) if rng.random() < 0.001 else rng.randrange(1 << 20) for _ in range(n)])
    print("\n".join(str(rng.randrange(1 << 30) | (rng.randrange(1 << 10) << 20)) for _ in range(q)))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, 15)
    q = rng.randint(1, 8)
    print(n, q)
    print(*[rng.randint(0, 100000) for _ in range(n)])
    for _ in range(q):
        print(rng.randint(0, 100000))
