import sys, random
if sys.argv[1] == "--max":  # sum n = 1e5: one n = 1e5 test, k = n / 3, values over all of 1..=1e9
    rng = random.Random(0); n = 10**5
    print(1); print(n, n // 3); print(*[rng.randint(1, 10**9) for _ in range(n)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n, r.randint(1, n)); print(*[r.randint(1, 10**9) for _ in range(n)])
