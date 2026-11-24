#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # sum x = sum y = 2e5 and t = 1e4: one x = 2e5 test whose |x - y| = 166320 has 160 divisors
    # (the longest output plus the divisor count), then 9999 x = 0 tests sharing the rest of y
    rest = 200_000 - 33_680
    tests = [(200_000, 33_680)] + [(0, rest // 9999 + (i < rest % 9999)) for i in range(9999)]
    print(len(tests))
    for x, y in tests:
        print(x, y)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    x = rng.randint(0, 20)
    y = rng.randint(0, 20)
    if x == 0 and y == 0: x = 1
    print(x, y)
