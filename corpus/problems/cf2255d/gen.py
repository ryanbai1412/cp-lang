import sys, random
if sys.argv[1] == "--max":  # one case n = 2e5 (sum n): values over every level, powers of two and one past them
    r = random.Random(0); n = 200000
    v = [r.choice([r.randint(1, 10**9), 1 << r.randint(0, 29), (1 << r.randint(0, 29)) + 1, 10**9]) for _ in range(n)]
    print(1); print(n); print(*v); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 8; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S); m = r.choice([3, 8, 40, 10**9]); print(n); print(*[r.choice([r.randint(1, m), 1 << r.randint(0, 4), (1 << r.randint(0, 4)) + 1]) for _ in range(n)])
