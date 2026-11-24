import sys, random
if sys.argv[1] == "--max":  # one case n = 2e5 (sum n), three rankings a few local swaps apart
    rng = random.Random(0); n = 200000; base = list(range(1, n + 1)); rng.shuffle(base)
    print(1); print(n)
    for _ in range(3):
        p = base[:]
        for _ in range(n // 3):
            i = rng.randrange(n); j = min(n - 1, i + rng.randint(0, 2)); p[i], p[j] = p[j], p[i]
        print(*p)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n); base = list(range(1, n + 1)); r.shuffle(base)
    for _ in range(3):
        p = base[:]
        for _ in range(r.randint(0, n)):
            i = r.randrange(n); j = min(n - 1, i + r.randint(0, 2)); p[i], p[j] = p[j], p[i]
        print(*p)
