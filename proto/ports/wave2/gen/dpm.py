import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); k = r.randint(0, 6 * S)
print(n, k); print(*[r.randint(0, k) for _ in range(n)])
