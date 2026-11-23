import random
random.seed(1)
n = 200000
print(1); print(n, random.randint(1, n)); print(*[random.randint(1, 3) for _ in range(n)])
