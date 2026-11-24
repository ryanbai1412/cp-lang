#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 1e3 and sum n = 3e5: one n = 299001 test with k = 2n - 1 (the longest braided block,
    # all 2n cards printed) and 999 n = 1 tests alternating k = 1 and k = 1e6 (NO)
    big = 300_000 - 999
    tests = [(big, 2 * big - 1)] + [(1, 1 if i % 2 else 10**6) for i in range(999)]
    print(len(tests))
    for n, k in tests:
        print(n, k)
    sys.exit()

# `seed size`: n <= size cards, k up to 2n + 2. size >= 3e5 fills sum n = 3e5 with k up to 1e6:
# one n = 3e5 test (k = n, 2n - 1, or 1e6) or 1000 tests of n = 300.
rng = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 12
N = 300_000
if S >= N:
    seed = int(sys.argv[1])
    tests = [(N, [N, 2 * N - 1, 10 ** 6, rng.randint(N, 2 * N - 1)][seed % 4])] if seed % 5 else \
        [(300, rng.choice([rng.randint(1, 10 ** 6), rng.randint(300, 599)])) for _ in range(1000)]
else:
    tests = []
    for _ in range(rng.randint(1, 8)):
        n = rng.randint(1, S)
        tests.append((n, rng.randint(1, min(10 ** 6, 2 * n + 2))))
print(len(tests))
for n, k in tests: print(n, k)
