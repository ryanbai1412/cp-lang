import sys, random
# `seed size`: up to 4 tests of 2 <= n <= 6 size^2, 1 <= k < n. size >= 2e5 fills sum n = 2e5:
# one n = 2e5 test (k = 1, n - 1, or random), 1e5 tests of n = 2, or 100 tests of n = 2000.
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
N = 2 * 10 ** 5
if S >= N:
    seed = int(sys.argv[1])
    if seed % 5 == 3: tests = [(2, 1)] * (N // 2)
    elif seed % 5 == 4: tests = [(2000, r.randint(1, 1999)) for _ in range(100)]
    else: tests = [(N, [1, N - 1, r.randint(1, N - 1)][seed % 5])]
else:
    tests = []
    for _ in range(r.randint(1, 4)):
        n = r.randint(2, max(2, 6 * S * S)); tests.append((n, r.randint(1, n - 1)))
print(len(tests))
for n, k in tests: print(n, k)
