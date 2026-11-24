import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); v = r.sample(range(1500000001), 10000); print(200000); print("\n".join(str(r.choice(v)) for _ in range(200000))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 4 * S); print(n)
for _ in range(n): print(r.choice([r.randint(0, 5), 1500000000, r.randint(0, 1500000000)]))
