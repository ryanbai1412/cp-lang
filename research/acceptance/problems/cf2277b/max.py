import random
random.seed(1)
tests = []
n = 100000; a = [random.randint(n, 10**9) for _ in range(n)]; a[random.randrange(n)] = n - 1; tests.append(a)
tests.append([50000] * 50000)
tests.append(list(range(10**9, 10**9 - 30000, -1)))
tests += [[random.choice([0, 1, 2, 10**9]) for _ in range(2)] for _ in range(9997)]
print(len(tests))
for a in tests: print(len(a)); print(*a)
