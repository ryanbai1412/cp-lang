import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(3, 3 * S + 3); c = r.randint(3, S + 4); print(n, c)
print(*[r.randrange(c) for _ in range(n)])
