#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # one test at n = 1e6 (sum n <= 1e6), values 0..=n: the first half is a staircase of prefix
    # minima (blocks of a falling minimum and a high value), the second half falls strictly so
    # every position there is a strict suffix maximum: the most candidates on both sides
    n = 10**6
    h = n // 2
    a = [n - (i // 2) if i % 2 else n - 2 * (i // 2) - 1 for i in range(h)]
    a += [n - 1 - 2 * k for k in range(n - h)]
    a = [max(0, x) for x in a]
    print(1); print(n); print(*a)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(2, 16)
    print(n)
    print(*[rng.randint(1, 20) for _ in range(n)])
