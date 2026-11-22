import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 2 * S); print(n)
M = r.choice([10, 100, 10**9])
for _ in range(n): print(r.randint(0, M), r.randint(1, 1000), r.randint(0, M // r.choice([1, 5, 50])))
