import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    m = r.randint(1, 2 * S + 2); print(m)
    k = r.randint(3, min(8, m + 3))
    for _ in range(m):
        a, b, c = r.sample(range(1, k + 1), 3)
        print(a, b, c)
