#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at sum n: waits near the 2e5 cap and keys almost everywhere keep about
    # sqrt(n + max a) key-count states alive at every door
    g = random.Random(1)
    n = 200_000
    print(1)
    print(n)
    print(*[g.randint(190_000, 200_000) for _ in range(n)])
    print("".join("1" if g.random() < 0.9 else "0" for _ in range(n)))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 14)
    print(n)
    print(*[rng.randint(0, 30) for _ in range(n)])
    print("".join(rng.choice("01") for _ in range(n)))
