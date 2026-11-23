import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(2, 5); a = sorted(random.sample(range(0, 14), n)); print(n, random.randint(0, min(n, 3))); print(*a)
