import sys, random
if sys.argv[1] == "--max":  # n = 1e4, m = 2e5 (input "n m r" then a_1..a_m), sizes over all of 0..n
    rng = random.Random(0); n, m = 10**4, 2 * 10**5
    print(n, m, rng.randint(0, n)); print(*[rng.randint(0, n) for _ in range(m)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); m = r.randint(0, 3 * S)
print(n, m); print(*[r.randint(0, n) for _ in range(m + 1)])
