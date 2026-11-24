import sys, random
# gen.py SEED SIZE | --max (n = 3000, k = 1500, entries up to 1e9)
if sys.argv[1] == "--max":
    r = random.Random(0); n = 3000; print(n, 1500); M = 10**9
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    n = r.randint(1, 3 * max(1, S)); print(n, r.randint(1, n)); M = r.choice([5, 10**9])
for i in range(n): print(*[r.randint(0, M) for _ in range(i + 1)])
