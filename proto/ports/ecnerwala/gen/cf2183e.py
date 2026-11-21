import sys, random

seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
full = S >= 8 and seed % 64 == 7
budget = 3000 if full else r.randint(2, 6 * S * S + 2)
cases = []
while budget >= 2:
    m = r.randint(2, budget) if full else r.randint(2, min(budget, 3 * S + 2))
    n = r.randint(2, m)
    budget -= m
    seq = [1]
    while len(seq) < n:
        x = seq[-1]
        ds = [d for d in range(1, x + 1) if x % d == 0 and x + d <= m]
        seq.append(x + r.choice(ds) if ds else r.randint(1, m))
    a = [v if r.random() < r.choice([0.0, 0.1, 0.3]) else 0 for v in seq]
    if r.random() < 0.2:
        a[r.randrange(n)] = r.randint(0, m)
    cases.append((n, m, a))
    if len(cases) == 1000:
        break
print(len(cases))
for n, m, a in cases:
    print(n, m)
    print(*a)
