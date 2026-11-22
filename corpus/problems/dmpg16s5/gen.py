import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 2 * S); print(n, r.randint(1, n)); M = r.choice([5, 10**9])
print(*[r.randint(-M, M) for _ in range(n)])
