import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 8; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S); m = r.choice([3, 8, 40, 10**9]); print(n); print(*[r.choice([r.randint(1, m), 1 << r.randint(0, 4), (1 << r.randint(0, 4)) + 1]) for _ in range(n)])
