import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S); print(n, r.randint(1, n)); M = r.choice([5, 10**9])
for i in range(n): print(*[r.randint(0, M) for _ in range(i + 1)])
