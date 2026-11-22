import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 1); k = r.randint(1, 10 * S)
print(n, k); print(*sorted(r.sample(range(1, 12 * S + 2), n)))
