import sys, random

if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 1e6): a random
    # permutation of 1..1e6.
    r = random.Random(0); n = 10**6; a = list(range(1, n + 1)); r.shuffle(a)
    print(1); print(n); print(*a)
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 4)
print(t)
for _ in range(t):
    n = r.randint(1, 5 * S)
    a = list(range(1, n + 1))
    r.shuffle(a)
    if r.random() < 0.3:
        b = 5 * r.randint(0, n // 5)
        a = [x for i in range(0, b, 5) for x in (i + 2, i + 1, i + 4, i + 3, i + 5)] + list(range(b + 1, n + 1))
        for _ in range(r.randint(0, 3)):
            i, j = r.randrange(n), r.randrange(n)
            a[i], a[j] = a[j], a[i]
    print(n)
    print(*a)
