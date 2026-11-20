import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, min(S + 2, 8)); print(n)
print(*(r.randint(1, n) for _ in range(n)))
