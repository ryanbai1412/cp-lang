import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 4 * S)
print(n); print(*[r.randint(1, r.choice([5, 10**9])) for _ in range(n)])
