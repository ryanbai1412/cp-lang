import sys, random
if sys.argv[1] == "--max":
    # one case n = 2e5 (sum n): cycles of every length 1, 2, ... plus many 2- and 3-cycles, so
    # many same-length cycles compete; b is a power of a with almost every entry hidden (YES).
    r = random.Random(0); n = 2 * 10**5
    lens, k, left = [], 1, n
    while left >= k and k <= 400: lens.append(k); left -= k; k += 1
    while left: d = min(left, r.choice([2, 3])); lens.append(d); left -= d
    perm = list(range(1, n + 1)); r.shuffle(perm)
    a, at = [0] * (n + 1), 0
    for d in lens:
        cyc = perm[at:at + d]; at += d
        for i in range(d): a[cyc[i]] = cyc[(i + 1) % d]
    b = list(range(1, n + 1))
    for _ in range(3): b = [a[x] for x in b]
    b = [x if r.random() < 0.02 else -1 for x in b]
    print(1); print(n); print(*a[1:]); print(*b); sys.exit()
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
