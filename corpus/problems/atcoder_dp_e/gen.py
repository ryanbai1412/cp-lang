import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S); cap = r.randint(1, 30)
print(n, cap)
for _ in range(n): print(r.randint(1, 12), r.randint(1, 9))
