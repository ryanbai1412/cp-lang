import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(2, 6 * S * S); print(n, r.randint(1, n - 1))
