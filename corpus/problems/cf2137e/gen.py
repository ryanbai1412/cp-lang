#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5 and k = 1e9: 0..n/2 once each (a high base), the rest duplicated
    # values above it.
    r = random.Random(2137)
    n, k = 200_000, 10**9
    a = list(range(n // 2)) + [r.randint(n // 2, n) for _ in range(n - n // 2)]
    r.shuffle(a)
    print(1)
    print(n, k)
    print(*a)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 40)
    k = r.randint(1, 5 if size == 0 else 10**9)
    print(n, k)
    print(*[r.randint(0, n) for _ in range(n)])
