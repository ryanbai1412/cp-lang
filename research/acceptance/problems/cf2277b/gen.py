import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(1, 5); hi = random.choice([1, n - 1, n, n + 1])
    lo = random.choice([0, max(0, n - 2), hi])
    a = [random.randint(min(lo, hi), max(lo, hi)) for _ in range(n)]
    if random.random() < 0.2: a = [a[0]] * n
    print(n); print(*a)
