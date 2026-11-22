import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 2 * S); M = r.choice([5, 10**6]); print(n, r.randint(1, M), r.randint(1, M))
print(*[r.randint(0, M) for _ in range(n)])
