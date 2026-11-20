import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); m = r.choice([5, 10**9]); print(n, r.randint(0, m)); print(*[r.randint(-m, m) for _ in range(n)])
