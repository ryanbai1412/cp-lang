import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); m = r.randint(0, 4 * S); print(n, m); p = r.choice([0.1, 0.5]); side = [r.randint(1, 2) for _ in range(n)]
    for _ in range(m):
        u, v = r.randint(1, n), r.randint(1, n)
        print(side[u - 1] if r.random() > p else r.randint(1, 2), u, v)
