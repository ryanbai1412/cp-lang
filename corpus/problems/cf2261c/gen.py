import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, S + 1); print(n); print(*[r.randint(0, r.choice([3, 100, 2**30 - 1])) for _ in range(n)])
