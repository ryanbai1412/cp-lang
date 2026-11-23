import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.randint(2, 8); print(n, random.randint(1, n))
    pool = list(range(1, n + 1)); nxt = n + 1
    for _ in range(n - 1):
        x, y = random.sample(pool, 2); pool.remove(x); pool.remove(y); print(x, y); pool.append(nxt); nxt += 1
