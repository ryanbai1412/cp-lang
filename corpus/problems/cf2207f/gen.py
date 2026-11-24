import sys, random

if sys.argv[1] == "--max":
    r = random.Random(1)
    n, m = 200_000, 1
    cards = [(rk, cl) for rk in range(1, n + 1) for cl in range(1, m + 1)]
    r.shuffle(cards)
    print(1)
    print(n, m)
    print(*[rk for rk, _ in cards])
    print(*[cl for _, cl in cards])
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 5)
print(t)
for _ in range(t):
    budget = max(1, 4 * S * S)
    n = r.randint(1, budget)
    m = r.randint(1, max(1, budget // n))
    if r.randint(0, 1):
        n, m = m, n
    cards = [(rk, cl) for rk in range(1, n + 1) for cl in range(1, m + 1)]
    r.shuffle(cards)
    print(n, m)
    print(*[rk for rk, _ in cards])
    print(*[cl for _, cl in cards])
