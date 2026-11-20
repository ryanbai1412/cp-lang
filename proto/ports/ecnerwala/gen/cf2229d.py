import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 3); print(n)
    p = list(range(1, 2 * n + 1)); r.shuffle(p)
    print(*p[:n])
    print(*p[n:])
