import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); p = r.random()
print(n)
for _ in range(n): print(*[int(r.random() < p) for _ in range(n)])
