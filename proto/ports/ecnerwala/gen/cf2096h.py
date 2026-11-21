import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, size + 2)
    m = rng.randint(1, min(6, size + 1))
    lim = (1 << m) - 1
    print(n, m)
    for _ in range(n):
        l = rng.randint(0, lim)
        r = rng.randint(l, lim)
        print(l, r)
