import random, sys
random.seed(int(sys.argv[1]))
V = 10 ** 18
t = random.randint(1, 5); print(t)
N = random.choice([8, 40, 300])
for _ in range(t):
    pick = random.random()
    if pick < 0.15: l = r = random.choice([0, 1, 2, 3, 4, 2 ** random.randint(1, 8), 2 ** random.randint(1, 8) - 1]) % (N + 1)
    elif pick < 0.25: l, r = 0, N
    elif pick < 0.55: l, r = sorted(random.randint(0, N) for _ in range(2))
    else:
        k = random.randint(2, 59)
        c = random.choice([random.randint(0, 5), random.randint(0, 3000)])
        l = random.choice([2 ** k - c, 2 ** k + c, random.randint(0, V), V - c])
        l = min(max(l, 0), V)
        r = min(l + random.choice([0, 1, random.randint(0, 200)]), V)
    print(l, r)
