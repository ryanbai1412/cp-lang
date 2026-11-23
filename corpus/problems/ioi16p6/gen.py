import sys, random
# IOI 2016 Aliens, sample-grader stdin: n m k, then n lines r c (0 <= r, c < m, 1 <= k <= n).
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S)
m = r.choice([1, 2, r.randint(1, 2 * S), r.randint(1, 10**6)])
k = r.choice([1, n, r.randint(1, n)])
print(n, m, k)
near = r.randint(0, 1) == 0
for _ in range(n):
    a = r.randrange(m)
    b = min(m - 1, max(0, a + r.randint(-3, 3))) if near else r.randrange(m)
    print(a, b)
