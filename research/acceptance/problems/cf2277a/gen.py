import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(1, 7); k = random.choice([1, 1, 2, random.randint(1, n)]); k = min(k, n)
    hi = random.choice([0, 1, 2, n])
    print(n, k); print(*[random.randint(0, hi) for _ in range(n)])
