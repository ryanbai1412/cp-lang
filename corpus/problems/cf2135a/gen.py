#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5: small values repeated in runs (many candidate blocks) mixed with
    # values across 1..n.
    r = random.Random(2135)
    n = 200_000
    print(1)
    print(n)
    print(*[r.randint(1, 20) if r.random() < 0.8 else r.randint(1, n) for _ in range(n)])
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    print(n)
    print(*[r.randint(1, min(n, 6)) for _ in range(n)])
