import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(1, 6); hi = random.choice((2, 6, 12, 12)); print(n)
    print(*[random.randint(1, hi) for _ in range(n)])
