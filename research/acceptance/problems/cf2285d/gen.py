import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.randint(1, 8); m = random.randint(1, n); q = random.randint(1, 4)
    print(n, m, q); print(*[random.randint(-2, 2) for _ in range(n)])
    for _ in range(q): print(random.randint(1, n), random.randint(-2, 2))
