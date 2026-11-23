import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    n = random.randint(2, 8); hi = random.choice((1, 2, 4, 10, 30, 10**6)); print(n)
    print(*[random.randint(1, hi) for _ in range(n)])
