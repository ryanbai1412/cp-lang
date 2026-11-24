import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); print(100); print(*[r.randint(130, 230) for _ in range(100)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); print(n)
print(*[r.randint(130, r.choice([140, 230])) for _ in range(n)])
