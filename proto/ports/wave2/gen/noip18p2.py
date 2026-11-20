import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 4); print(t)
for _ in range(t):
    n = r.randint(1, S + 1); print(n)
    print(*r.sample(range(1, 6 * S + 8), n))
