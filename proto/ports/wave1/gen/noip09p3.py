import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(1, 2 * S + 1); print(n, m)
print(*[r.randint(1, 100) for _ in range(n)])
for _ in range(m): print(r.randint(1, n), r.randint(1, n), r.randint(1, 2))
