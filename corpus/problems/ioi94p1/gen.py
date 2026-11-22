import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); print(n)
for i in range(1, n + 1): print(*[r.randint(0, 99) for _ in range(i)])
