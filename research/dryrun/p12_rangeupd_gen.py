import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); q = random.randint(1, s); big = random.random() < .3
v = lambda: random.randint(1, 10**6 if big else 9)
print(n, q); print(*[v() for _ in range(n)])
for _ in range(q):
    k = random.randint(1, 3); a, b = sorted([random.randint(1, n), random.randint(1, n)])
    print(k, a, b, *([v()] if k < 3 else []))
