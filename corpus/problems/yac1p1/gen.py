import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); top = r.choice([5, 10 ** 9]); print(n, r.randint(0, r.choice([10, 10 ** 9])))
for _ in range(2): print(*[r.randint(1, top) for _ in range(n)])
