import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    n = random.randint(1, 9); p = list(range(1, n + 1)); random.shuffle(p); print(n); print(*p)
