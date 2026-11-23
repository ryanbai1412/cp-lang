import random
random.seed(1)
print(2)
for n, k in ((100000, 50), (3333, 1500)):
    print(n, k); print(*[random.randint(1, 10**9) if random.random() < 0.3 else 5 + 3 * i for i in range(n)])
