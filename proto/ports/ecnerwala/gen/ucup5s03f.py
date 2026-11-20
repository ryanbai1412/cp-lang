import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); m = r.randint(0, 3 * S)
print(n, m); print(*[r.randint(0, n) for _ in range(m + 1)])
