import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); hi = r.choice([3, 15, 2**25 - 1]); base = r.choice([0, hi])
    print(n); print(*[(r.randint(0, hi) | (base if r.random() < 0.7 else 0)) for _ in range(n)])
