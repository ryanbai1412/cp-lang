import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.randint(2, 9); k = random.choice([1, n - 1, random.randint(1, n - 1)])
    kind = random.randint(0, 3)
    if kind == 0: a = [random.choice([1, 10**9])] * n
    elif kind == 1: a = [random.randint(1, 3) for _ in range(n)]
    elif kind == 2: a = [random.choice([1, 10**9, 10**9 - 1]) for _ in range(n)]
    else: a = [random.randint(1, 10**9) for _ in range(n)]
    print(n, k); print(*a)
