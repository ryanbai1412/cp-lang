import sys, random

if sys.argv[1] == "--max":
    # every character free: the most reachable ends and distinct scores
    n = 100_000
    print(1)
    print(n, 999_983, 1_000_000)
    print("?" * n)
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S >= 1000:
    tests = [100_000]
else:
    tests = [r.randint(1, 3 * S + 2) for _ in range(r.randint(1, 6))]
print(len(tests))
for n in tests:
    hi = r.choice([2, 5, 10**6])
    x, y = r.randint(1, hi), r.randint(1, hi)
    q = r.choice([0.0, 0.1, 0.5, 0.9, 1.0])
    s = "".join("?" if r.random() < q else r.choice("01") for _ in range(n))
    print(n, x, y)
    print(s)
