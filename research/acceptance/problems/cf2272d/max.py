import random
random.seed(1)
k = 1000
print(1); print(k)
for _ in range(k):
    print(200)
    for v in range(2, 201): print(random.randint(1, v - 1), v)
