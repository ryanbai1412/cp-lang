import sys, random
if sys.argv[1] == "--max":  # one case n = 2e5 (sum n), values over all 18 bits
    r = random.Random(0); n = 200000
    print(1); print(n); print(*[r.randrange(2**18) for _ in range(n)]); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); hi = r.choice([3, 15, 255, 2 * 10**5]); print(n); print(*[r.randint(0, hi) for _ in range(n)])
