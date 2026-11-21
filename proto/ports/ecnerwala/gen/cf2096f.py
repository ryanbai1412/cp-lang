import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 3)
    m = rng.randint(1, size * 3 + 3)
    print(n, m)
    for _ in range(m):
        t = rng.randint(0, 1)
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(t, l, r)
    q = rng.randint(1, size * 2 + 2)
    print(q)
    for _ in range(q):
        l = rng.randint(1, m)
        r = rng.randint(l, m)
        print(l, r)
