import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 2 * S + 1); m = r.randint(0, 2 * S + 1); k = r.choice([1, 2, n])
print(n, m); print(*[r.randint(max(1, v - k), v - 1) for v in range(2, n + 1)])
