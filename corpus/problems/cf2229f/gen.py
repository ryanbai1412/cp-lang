#!/usr/bin/env python3
"""CF 2229F: 1 <= k <= n <= 18, 1 <= a_i <= 1e9, sum of 2^n <= 2^18."""
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
M = 10 ** 9
if size >= 1000:
    mode = int(sys.argv[1]) % 5
    if mode == 0:
        tests = [(18, r.randint(1, 18), [M] * 18)]
    elif mode == 1:
        tests = [(18, k, [r.randint(M - 1000, M) for _ in range(18)]) for k in [r.randint(2, 9)]]
    elif mode == 2:
        tests = [(18, 1, [r.randint(1, M) for _ in range(18)])]
    elif mode == 3:
        tests = [(17, r.randint(1, 17), [r.randint(1, M) for _ in range(17)]) for _ in range(2)]
    else:
        tests = [(4, r.randint(1, 4), [r.randint(1, M) for _ in range(4)]) for _ in range(10000)]
else:
    tests = []
    for _ in range(r.randint(1, 3)):
        n = r.randint(1, 8)
        tests.append((n, r.randint(1, n), [r.randint(1, r.choice([5, 30, M])) for _ in range(n)]))
print(len(tests))
for n, k, a in tests:
    print(n, k)
    print(*a)
