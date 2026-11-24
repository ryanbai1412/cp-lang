#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5 and the target at the edge of what n expansions can reach.
    rng = random.Random(2157)
    n = 200_000
    s = "".join(rng.choice("48") for _ in range(n))
    print(1)
    print(n, n - 3 * n // 4 // 2, -(3 * n // 4))
    print(s)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, max(2, size * 5))
    x = rng.randint(-n - 2, n + 2)
    y = rng.randint(-n - 2, n + 2)
    s = "".join(rng.choice("48") for _ in range(n))
    print(n, x, y)
    print(s)
