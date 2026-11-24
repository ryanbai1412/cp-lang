"""gen.py SEED SIZE | --max: NOI 2008 Olympic Logistics. m <= n <= 60, 0.3 <= k < 1, successors
S_i != i with every station reaching station 1, and positive reals C_i <= 1e6. Small sizes
keep n small with any m; --max is n = m = 60 on a deep tree hanging off a long cycle through 1."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 60 if MAX else r.randint(2, min(60, 2 + S))
m = n if MAX else r.randint(0, n)
k = 0.99 if MAX else r.choice([0.3, 0.5, 0.99, round(r.uniform(0.3, 0.99), 2)])
cyc = r.randint(2, n // 2 if MAX else n)
order = [1] + r.sample(range(2, n + 1), n - 1)
s = [0] * (n + 1)
for i in range(cyc):
    s[order[i]] = order[(i + 1) % cyc]
for i in range(cyc, n):
    s[order[i]] = order[i - 1] if MAX and r.random() < 0.8 else order[r.randrange(i)]
c = [round(r.uniform(0.01, 1e6), 2) if r.random() < 0.7 else float(r.randint(1, 10)) for _ in range(n)]
print(n, m, k)
print(*s[1:])
print(*c)
