import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(2, 6); print(n); print(*[random.randint(-3, 3) for _ in range(n)])
