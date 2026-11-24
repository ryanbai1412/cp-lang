import sys, random
if sys.argv[1] == "--max":
    # sum m and sum k saturated by one test: all 8m states with k = 5e5
    g = random.Random(1)
    m = 500_000
    a, b, c = sorted(g.sample(range(1, m), 3))
    print(1)
    print(a, b, c, m, 500_000)
    sys.exit()


seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
budget_m = budget_k = 500_000 if full else r.randint(4, 40 * S * S)
cases = []
while budget_m >= 4 and budget_k >= 1 and len(cases) < 10_000:
    m = budget_m if full and not cases else r.randint(4, min(budget_m, 6 * S * S + 4))
    a, b, c = sorted(r.sample(range(1, m), 3))
    k = r.randint(1, min(budget_k, r.choice([3, m, 2 * m])))
    budget_m -= m
    budget_k -= k
    cases.append((a, b, c, m, k))
print(len(cases))
for case in cases:
    print(*case)
