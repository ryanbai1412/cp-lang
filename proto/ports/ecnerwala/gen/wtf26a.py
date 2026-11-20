import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(2, 2 * S + 2); print(n)
    wmax = r.choice([10, 1000, 10**9])
    for v in range(2, n + 1):
        print(r.randint(1, v - 1), v, r.randint(1, wmax))
