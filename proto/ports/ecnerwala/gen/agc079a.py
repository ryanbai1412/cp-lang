import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 1); print(n); print(*[r.randint(1, i) for i in range(1, n)])
