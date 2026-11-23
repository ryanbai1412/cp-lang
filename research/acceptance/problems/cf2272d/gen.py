import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    k = random.randint(2, 3); print(k)
    for _ in range(k):
        n = random.randint(1, 3); print(n)
        for v in range(2, n + 1): print(random.randint(1, v - 1), v)
