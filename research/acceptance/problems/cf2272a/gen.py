import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(2, 7); a = list(range(n)); random.shuffle(a); print(n, random.randint(1, n)); print(*a)
