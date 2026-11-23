import random
random.seed(1)
print(3)
for n, k in ((100000, 50), (3333, 1500)):
    print(n, k); print(*[random.randint(1, 10**9) if random.random() < 0.3 else 5 + 3 * i for i in range(n)])
# run-6 agent's case: a_i = i^2 + 1 keeps every chain alive; a per-step list copy is 25 s here
print(6666, 1500); print(*[i * i + 1 for i in range(6666)])
