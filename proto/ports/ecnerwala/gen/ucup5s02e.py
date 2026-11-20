import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n); base = list(range(1, n + 1)); r.shuffle(base)
    for _ in range(3):
        p = base[:]
        for _ in range(r.randint(0, n)):
            i = r.randrange(n); j = min(n - 1, i + r.randint(0, 2)); p[i], p[j] = p[j], p[i]
        print(*p)
