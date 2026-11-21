import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S >= 1000:
    tests = [S]
else:
    tests = [r.randint(1, 3 * S + 2) for _ in range(r.randint(1, 6))]
print(len(tests))
for n in tests:
    hi = r.choice([1, 2, 3, n, 10**9])
    a = [r.randint(0, min(hi, 10**9)) for _ in range(n)]
    if r.random() < 0.5:
        a = sorted(a, reverse=True)
        for _ in range(r.randint(0, 2)):
            i, j = r.randrange(n), r.randrange(n)
            a[i], a[j] = a[j], a[i]
    print(n)
    print(*a)
