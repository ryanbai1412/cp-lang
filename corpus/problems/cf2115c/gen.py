#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # sum n = 100 as five tests of n = 20, m = 4000, every hp 400 (s0 n + k0 is largest)
    ps = (50, 37, 63, 1, 99)
    print(len(ps))
    for p in ps:
        print(20, 4000, p); print(*[400] * 20)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, 6)
    m = rng.randint(1, 14)
    p = rng.randint(0, 100)
    print(n, m, p)
    print(*[rng.randint(1, 6) for _ in range(n)])
