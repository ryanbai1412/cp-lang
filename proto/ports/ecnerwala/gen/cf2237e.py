import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 2 * S + 2); print(n)
    a = list(range(1, n + 1)); r.shuffle(a)
    if r.random() < 0.5:
        # Build a commuting-ish b with some positions hidden
        b = list(range(1, n + 1)); r.shuffle(b)
        b = [x if r.random() < 0.35 else -1 for x in b]
    else:
        # Constructive commuting b = a^k then mask some entries
        k = r.randint(0, n)
        b = list(range(1, n + 1))
        for _ in range(k): b = [a[x - 1] for x in b]
        b = [x if r.random() < 0.5 else -1 for x in b]
    print(*a)
    print(*b)
