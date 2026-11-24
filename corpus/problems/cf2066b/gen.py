import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5: 1..n-2 falling with two zeros, so the kept zero is checked
    # against suffix mexes that climb to n
    rng = random.Random(1)
    n = 200_000
    a = list(range(n - 2, 0, -1))
    for _ in range(2):
        a.insert(rng.randrange(len(a) + 1), 0)
    a[rng.randrange(n)] = 10**9
    print(1)
    print(n)
    print(*a)
    sys.exit()

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
