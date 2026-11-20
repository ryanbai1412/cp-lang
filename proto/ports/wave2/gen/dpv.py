import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 5 * S); p = r.sample(range(1, n + 1), n)
print(n, r.randint(2, r.choice([3, 10, 10**9])))
for i in range(1, n): print(p[i], p[r.randint(max(0, i - r.choice([1, n])), i - 1)])
