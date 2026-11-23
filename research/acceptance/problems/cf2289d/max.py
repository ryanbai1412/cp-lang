import random
random.seed(1)
n = 200000
print(1); print(n); print(*[random.randint(-10**9, 10**9) for _ in range(n)])
