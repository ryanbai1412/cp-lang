import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 1); m = r.randint(0, 3 * S) if n > 1 else 0
    print(n, m)
    for _ in range(m):
        u = r.randint(1, n); v = r.randint(1, n)
        while v == u: v = r.randint(1, n)
        if r.random() < 0.6 and u > v: u, v = v, u
        print(u, v)
