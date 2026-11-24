import sys, random
if sys.argv[1] == "--max":  # sum n = 2e5: one n = 2e5 test with k = n ^ 5 (a permutation is printed)
    print(1); print(200000, 200000 ^ 5); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 8; print(t)
for _ in range(t):
    n = r.choice([r.randint(1, 4 * S), 1 << r.randint(0, 5)]); print(n, r.randint(0, 2 * n + 2))
