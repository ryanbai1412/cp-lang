import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 3); print(n)
for v in range(2, n + 1): print(r.randint(1, v - 1), v)
print(*[r.randint(1, 10000) for _ in range(n)])
