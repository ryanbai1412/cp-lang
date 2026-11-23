import random
random.seed(1)
n = 200000
print(1); print(n, n // 3); print(*sorted(random.sample(range(10**9 + 1), n)))
