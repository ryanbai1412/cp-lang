import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 2); m = r.randint(1, 3 * S); print(n, m)
for _ in range(m):
    a, b = r.sample(range(1, n + 1), 2); print(a, b, r.randint(1, r.choice([5, 10**9])))
