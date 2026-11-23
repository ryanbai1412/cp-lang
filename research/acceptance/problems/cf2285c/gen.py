import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(1, 4); print(n); print(*[random.randint(1, 5) for _ in range(n)])
