import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 8; print(t)
for _ in range(t):
    n = r.choice([r.randint(1, 4 * S), 1 << r.randint(0, 5)]); print(n, r.randint(0, 2 * n + 2))
