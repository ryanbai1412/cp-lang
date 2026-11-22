import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); print(n); print(*[r.randint(1, 20000) for _ in range(n)])
