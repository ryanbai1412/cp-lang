import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(4, 4 * S + 4); h = n // 2
    if r.random() < 0.7:
        d = r.randint(0, 50); M = r.choice([d * (h - 1) + r.randint(0, d), r.randint(d * (n - h - 1), d * (n - h - 1) + 100), d * r.randint(0, n)])
        M = max(M, d * (n - h - 1))
        a = sorted([d * i for i in range(h)] + [M - d * i for i in range(n - h)])
        if a[0] != 0: a = [x - a[0] for x in a]
        if r.random() < 0.15: i = r.randrange(1, n); a[i] += r.randint(0, 1); a = [0] + sorted(a[1:])
    else:
        a = [0] + sorted(r.randint(0, 30) for _ in range(n - 1))
    print(n); print(*a)
