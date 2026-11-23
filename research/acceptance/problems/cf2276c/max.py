import random
random.seed(1)
tests = [(5, random.randint(1, 4), [random.randint(1, 10**9) for _ in range(5)]) for _ in range(9000)]
tests.append((100000, 1, [10**9] * 100000))
tests.append((20000, 2, [1] * 20000))
tests.append((20000, 19999, [random.randint(1, 10**9) for _ in range(20000)]))
tests.append((15000, 7500, sorted(random.randint(1, 10**9) for _ in range(15000))))
print(len(tests))
for n, k, a in tests:
    print(n, k); print(*a)
