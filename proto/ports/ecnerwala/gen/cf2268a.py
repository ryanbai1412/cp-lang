import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); print(n, r.randint(1, n)); print(*[r.randint(1, 10**9) for _ in range(n)])
