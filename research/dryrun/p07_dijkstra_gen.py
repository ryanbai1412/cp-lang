import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(2, max(2, s)); M = random.choice([3, 1000, 10**9])
es = [(random.randint(1, i - 1), i) for i in range(2, n + 1)]
es += [(random.randint(1, n), random.randint(1, n)) for _ in range(random.randint(0, 2 * s))]
random.shuffle(es)
print(n, len(es))
for a, b in es: print(a, b, random.randint(1, M))
