#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # sum n m = 2.5e5 in one 500 x 500 grid (the O(n^2 m) worst), 70% ones so most row pairs
    # share many columns.
    rng = random.Random(2159)
    n = m = 500
    print(1)
    print(n, m)
    print("\n".join("".join("1" if rng.random() < 0.7 else "0" for _ in range(m)) for _ in range(n)))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 8)
    m = rng.randint(2, 8)
    print(n, m)
    for _ in range(n):
        print("".join(rng.choice("01") for _ in range(m)))
