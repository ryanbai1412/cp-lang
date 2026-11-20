import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    big = r.random() < 0.2
    n = r.randint(2, 10**9 if big else 2 + 2 * S); q = 6
    print(n, r.randint(1, n), r.randint(1, n), q)
    for _ in range(q):
        k = 1 if big else n   # keep c = -(a*x + b*y) inside 32 bits
        a, b = r.randint(-k, k), r.randint(-k, k)
        if a == 0 and b == 0: a = 1
        if r.random() < 0.5: a, b = r.choice([(1, 1), (1, -1), (-1, 1), (0, 1), (1, 0), (2, -2), (a, b)])
        x, y = r.randint(1, n), r.randint(1, n)
        c = -(a * x + b * y) if r.random() < 0.8 else r.randint(-n, n)
        print(a, b, c, r.randint(0, 10**9 if big else 5 * n))
