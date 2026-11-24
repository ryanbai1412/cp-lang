import sys, random
if sys.argv[1] == "--max":  # one case n = 2e5 (sum n), ratings over all of 0..1e9
    rng = random.Random(0); n = 200000
    print(1); print(n); print(*[rng.choice([rng.randint(0, 10**9), 10**9, 0]) for _ in range(n)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, S + 1); print(n); print(*[r.randint(0, r.choice([3, 100, 2**30 - 1])) for _ in range(n)])
