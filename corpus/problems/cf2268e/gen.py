import sys, random
if sys.argv[1] == "--max":  # sum n = 2e5: one n = 2e5 case, a_i over all of [0, 2^18)
    rng = random.Random(0); n = 200000
    print(1); print(n); print(*[rng.randrange(1 << 18) for _ in range(n)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(2, 4 * S * S); hi = r.choice([1, 3, 7, 2 ** 18 - 1])
    print(n); print(*[r.randint(1, hi) for _ in range(n)])
