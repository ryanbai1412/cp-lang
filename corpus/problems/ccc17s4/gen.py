import sys, random
MAX = sys.argv[1] == "--max"  # n = 1e5, m = 2e5, costs up to 1e9
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = r.randint(1, S + 1); m = n - 1 + r.randint(0, 2 * S); M = r.choice([3, 6, 10**9])
if MAX: n, m, M = 10**5, 2 * 10**5, 10**9
if m == 0: n = 2; m = 1
print(n, m, r.randint(0, M))
for i in range(2, n + 1): print(r.randint(1, i - 1), i, r.randint(1, M))
for _ in range(m - (n - 1)): print(r.randint(1, n), r.randint(1, n), r.randint(1, M))
