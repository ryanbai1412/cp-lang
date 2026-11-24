import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); print(100); print(*[r.randint(1, 1000) for _ in range(100)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(4, S + 5); print(n)
print(*[r.randint(1, 1000) for _ in range(n)])
