import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 5 * S)
print(n); print(*[r.randint(1, r.choice([9, 10**9])) for _ in range(n)])
