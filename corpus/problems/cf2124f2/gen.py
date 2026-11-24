import sys, random
if sys.argv[1] == "--max":
    # one test at n = m = 5000 (both sums): half the restrictions near the main diagonals, where
    # the runs and wraps live, the rest anywhere
    rng = random.Random(0)
    n = m = 5000
    pairs = set()
    while len(pairs) < m // 2:
        i = rng.randint(1, n); x = i - rng.randint(-3, 40)
        if 1 <= x <= n: pairs.add((i, x))
    while len(pairs) < m:
        pairs.add((rng.randint(1, n), rng.randint(1, n)))
    print(1); print(n, m)
    for i, x in sorted(pairs): print(i, x)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 2)
    m = rng.randint(0, min(n * n, size * 4))
    pairs = set()
    while len(pairs) < m:
        pairs.add((rng.randint(1, n), rng.randint(1, n)))
    print(n, len(pairs))
    for i, x in sorted(pairs):
        print(i, x)
