import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); print(n); x = r.randrange(n)
    print(*[(x // i if r.random() < 0.8 else r.randint(0, n // i)) for i in range(1, n + 1)])
