import random
random.seed(1)
tests = [random.sample(range(5 * 10**8, 10**9 + 1), 3000), [k * 10**6 for k in range(1, 1001)], [10**9] * 500]
tests += [[1]] * 500
print(len(tests))
for a in tests: print(len(a)); print(*a)
