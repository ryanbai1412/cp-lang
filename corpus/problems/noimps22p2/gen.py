import sys, random
# seed size: n <= size nodes, l <= r and K bounded by 10, 500, 2e5 or 1e9, a random tree or the path
# 1-2-...-n; --max: n = 200, a random tree, r and K up to 1e9 with wide ranges
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 200 if mx else r.randint(1, min(200, S))
V = 10**9 if mx else r.choice([10, 500, 2 * 10**5, 10**9])
K = r.randint(V // 2, V) if mx else r.randint(1, V)
print(n, K)
for _ in range(n):
    a, b = sorted(r.randint(1, V) for _ in range(2))
    print(a, b)
chain = not mx and r.random() < 0.3
for i in range(2, n + 1):
    print(i - 1 if chain else r.randint(1, i - 1), i)
