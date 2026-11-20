import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(2, 3 * S + 1); q = r.randint(0, 3 * S); print(n, q); print(*[r.randint(0, 2**30 - 1) for _ in range(n)])
    for _ in range(q): print(r.randint(1, n), r.randint(0, 2**30 - 1))
