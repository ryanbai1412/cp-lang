#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5: a slowly rising a with plateaus (many prefix maxima, long stretches
    # under one), b equal to a half the time.
    r = random.Random(2137)
    n = 200_000
    a = [min(2 * n, max(1, i * 2 * n // n - r.randint(0, 2 * n // 3))) for i in range(n)]
    b = [x if r.random() < 0.5 else r.randint(1, 2 * n) for x in a]
    print(1)
    print(n)
    print(*a)
    print(*b)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    print(n)
    print(*[r.randint(1, 2 * n) for _ in range(n)])
    print(*[r.randint(1, 2 * n) for _ in range(n)])
