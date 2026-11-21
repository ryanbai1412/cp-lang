import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = 4
print(t)
for _ in range(t):
    n = r.randint(1, 4 * S + 3)
    m = r.randint(1, 6 * S + 4)
    q = r.randint(1, n)
    print(n, m, q)
    max_w = r.choice([0, 1, min(m, 3), min(m, 8), m])
    for _ in range(m):
        u = r.randint(1, n)
        v = r.randint(1, n)
        w = r.randint(0, max_w)
        print(u, v, w)
    print(*(r.randint(1, n) for _ in range(q)))
