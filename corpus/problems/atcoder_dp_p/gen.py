import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    n = 100000
    print(n)
    for i in range(1, n): print(i, i + 1)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 5 * S); p = r.sample(range(1, n + 1), n)
print(n)
for i in range(1, n): print(p[i], p[r.randint(max(0, i - r.choice([1, n])), i - 1)])
