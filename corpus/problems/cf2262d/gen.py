import sys, random
if sys.argv[1] == "--max":  # sum n^2 = 1e6: one case n = 1000, a = 1 and b = n everywhere (~n^2 ops)
    n = 1000; print(1); print(n); print(*[1] * n); print(*[n] * n); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 1); mode = r.randrange(3)
    b = [r.randint(i + 1 if mode else 1, n) if i + 1 < n else n for i in range(n)]
    if mode == 2: b = [min(n, i + 1 + r.randint(0, 2)) for i in range(n)]
    a = [r.randint(1, x) if r.random() < 0.5 else x for x in b]
    if r.random() < 0.1: a[r.randrange(n)] = r.randint(1, n)
    print(n); print(*a); print(*b)
