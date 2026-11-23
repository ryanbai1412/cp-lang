import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    n = random.randint(1, 9); print(n, random.randint(1, n)); print(*[random.randint(1, random.randint(1, n)) for _ in range(n)])
