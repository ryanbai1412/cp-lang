import sys, random
if sys.argv[1] == "--max":
    # one n = 1e5 test (sum n) with values near 1e9, so the sum reaches 1e14
    g = random.Random(1)
    n = 10**5
    print(1)
    print(n)
    print(*(g.randint(10**9 - 10**6, 10**9) for _ in range(n)))
    print(*(g.randint(10**9 - 10**6, 10**9) for _ in range(n)))
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S + 2); print(n)
    print(*(r.randint(1, 10**9) for _ in range(n)))
    print(*(r.randint(1, 10**9) for _ in range(n)))
