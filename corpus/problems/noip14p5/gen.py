import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.randint(0, 3 * n)
print(n, m)
for _ in range(m): print(r.randint(1, n), r.randint(1, n))
print(r.randint(1, n), r.randint(1, n))
