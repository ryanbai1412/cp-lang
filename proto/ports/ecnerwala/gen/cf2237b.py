import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 3); print(n)
    b = sorted(r.randint(1, 3 * n) for _ in range(n))
    a = [r.randint(1, 3 * n + 2) for _ in range(n)]
    print(*a)
    print(*b)
