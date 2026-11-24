import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 5 * 10**5
n = 10**6 if big else r.randint(1, 2 * S); M = 10**6 if big else r.choice([5, 10**6]); print(n, r.randint(1, M), r.randint(1, M))
print(*[r.randint(0, M) for _ in range(n)])
