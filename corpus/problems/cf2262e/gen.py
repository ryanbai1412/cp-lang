import sys, random
if sys.argv[1] == "--max":  # sum n <= 500: one case n = 500, M the largest prime <= 1e9
    print(1); print(500, 999999937); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 3); print(t)
for _ in range(t):
    print(r.randint(1, 5 * S), r.choice([998244353, 10 ** 9 + 7, 10 ** 9 + 9, 999999937]))
