import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S); m = r.randint(1, S); p = r.randint(1, 2 * S); q = r.randint(1, 2 * S); print(n, m, p, q)
M = r.choice([3, 10**8])
for _ in range(p): print(r.randint(1, m), r.randint(1, m), r.randint(1, M))
for _ in range(q): print(r.randint(1, n), r.randint(1, n), r.randint(1, M))
