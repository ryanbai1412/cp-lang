import sys, random
if sys.argv[1] == "--max":  # sum n = 2e5: one n = 2e5 test, piles near 1e9 in falling order (every pile merges)
    rng = random.Random(0); n = 200000
    print(1); print(n); print(*sorted((rng.randint(10**9 // 2, 10**9) for _ in range(n)), reverse=True)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S + 2); print(n)
    print(*(r.randint(1, 10**9) for _ in range(n)))
