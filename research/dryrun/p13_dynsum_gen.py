import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); q = random.randint(1, s)
v = lambda: random.randint(1, 10**9)
print(n, q); print(*[v() for _ in range(n)])
for _ in range(q):
    if random.random() < .5: print(1, random.randint(1, n), v())
    else: print(2, *sorted([random.randint(1, n), random.randint(1, n)]))
