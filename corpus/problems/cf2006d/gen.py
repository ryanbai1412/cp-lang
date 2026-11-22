import sys, random

if sys.argv[1] == "--max":
    r = random.Random(1)
    n, q, k = 10**5, 10**5, 10**6
    print(1)
    print(n, q, k)
    print(*[r.randint(1, k) for _ in range(n)])
    for _ in range(q):
        l = r.randint(1, n - 1)
        print(l, r.randint(l + 1, n))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 4)
print(t)
for _ in range(t):
    n = r.randint(2, max(2, S))
    q = r.randint(1, max(1, S))
    k = r.choice([1, 2, 10, 100, r.randint(1, 10**6)])
    print(n, q, k)
    print(*[r.randint(1, k) for _ in range(n)])
    for _ in range(q):
        l = r.randint(1, n - 1)
        print(l, r.randint(l + 1, n))
