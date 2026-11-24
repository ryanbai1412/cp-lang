import sys, random
# seed size: T <= 3 tests of N <= size + 1 sentences (no spaces, length <= 30); --max: one test, N = 1e5
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 10**5 if mx else int(sys.argv[2])
tests = 1 if mx else r.randint(1, 3)
print(tests)
for _ in range(tests):
    n = 10**5 if mx else r.randint(1, min(10**5, S + 1))
    hi = 30 if mx or r.random() < 0.5 else r.randint(1, 5)
    l = r.randint(1, 3 * 10**6) if not mx and r.random() < 0.2 else r.randint(1, 31 * min(n, 6))
    p = 10 if mx else r.randint(1, 10)
    print(n, l, p)
    for _ in range(n):
        print("".join(r.choice("abcxyz.,!?") for _ in range(r.randint(1, hi))))
