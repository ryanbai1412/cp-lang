import random
random.seed(1)
t = 10000
print(t); print(*[random.randint(10**17, 10**18) for _ in range(t)], sep="\n")
