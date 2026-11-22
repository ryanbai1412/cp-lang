import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); m = r.randint(0, 2 * S); print(n, m)
    for _ in range(m):
        l = r.randint(1, n); print(l, r.randint(l, n))
