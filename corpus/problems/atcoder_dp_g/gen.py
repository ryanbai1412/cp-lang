import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 2); m = r.randint(1, 2 * S)
print(n, m)
seen = set()
for _ in range(m):
    u = r.randint(1, n - 1); v = r.randint(u + 1, n)
    print(u, v)
