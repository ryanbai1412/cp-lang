import random
random.seed(1)
tests = []
# many tiny tests: per-test overhead (t = 1e4 with resets)
tests += [[random.randint(1, 10**6) for _ in range(10)] for _ in range(4990)]
# one bad gap in the middle, every other gap tiny: the doubled run is the whole array, worst for per-l rescans
m = 50000
tests.append([500000 + (i % 2) for i in range(m // 2)] + [250000 + (i % 2) for i in range(m - m // 2)])
tests.append([10**6] * 50000)
tests.append(sorted(random.randint(1, 10**6) for _ in range(50000)))
tests.append([random.randint(1, 10**6) for _ in range(50000)])
# two bad gaps with a small-gap run between them
tests.append([1] * 16000 + [300000 + i % 3 for i in range(18000)] + [10**6] * 16000)
assert len(tests) <= 10**4 and sum(map(len, tests)) <= 3 * 10**5
print(len(tests))
for a in tests: print(len(a)); print(*a)
