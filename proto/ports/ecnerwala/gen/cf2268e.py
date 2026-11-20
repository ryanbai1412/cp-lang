import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(2, 4 * S * S); hi = r.choice([1, 3, 7, 2 ** 18 - 1])
    print(n); print(*[r.randint(1, hi) for _ in range(n)])
