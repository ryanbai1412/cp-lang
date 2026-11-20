import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(2, 3 * S + 2)
    m = r.randint(1, n)
    print(n, m)
    print(*(r.randint(1, m) for _ in range(n)))
