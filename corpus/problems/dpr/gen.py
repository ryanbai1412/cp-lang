import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1)
print(n, r.randint(1, r.choice([5, 10**18])))
for _ in range(n): print(*[r.randint(0, 1) for _ in range(n)])
