import sys, random
if sys.argv[1] == "--max":
    # one test at sum n, |a_i| = 1e9 scale with both signs and a mid k: the Aliens search runs
    # its full log C rounds of Monge paths
    g = random.Random(1)
    n = 200_000
    a = [g.randint(1, 10**9) * (-1 if g.random() < 0.3 else 1) for _ in range(n)]
    print(1)
    print(n, n // 3)
    print(*a)
    sys.exit()


r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S >= 1000:
    tests = [200_000]
else:
    tests = [r.randint(1, 2 * S + 3) for _ in range(r.randint(1, 5))]
print(len(tests))
for n in tests:
    hi = r.choice([3, 20, 10**9])
    neg = r.choice([0.0, 0.2, 0.5, 0.8, 1.0])
    a = [r.randint(1, hi) * (-1 if r.random() < neg else 1) for _ in range(n)]
    if r.random() < 0.2:
        a = [x if r.random() < 0.7 else 0 for x in a]
    print(n, r.choice([1, n, r.randint(1, n)]))
    print(*a)
