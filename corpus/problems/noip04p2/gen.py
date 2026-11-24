import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); print(10000); print(*[r.randint(1, 20000) for _ in range(10000)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); print(n); print(*[r.randint(1, 20000) for _ in range(n)])
