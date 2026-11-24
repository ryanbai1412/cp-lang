import sys, random
if sys.argv[1] == "--max":  # sum n = 2e5: one n = 2e5 test, |a_i| <= 1e9, c = 1e9 / 2
    rng = random.Random(0); n = 200000
    print(1); print(n, 5 * 10**8); print(*[rng.randint(-10**9, 10**9) for _ in range(n)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); m = r.choice([5, 10**9]); print(n, r.randint(0, m)); print(*[r.randint(-m, m) for _ in range(n)])
