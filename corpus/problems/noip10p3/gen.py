import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0); n, m = 20000, 100000; print(n, m); seen = set()
    while len(seen) < m:
        a, b = r.sample(range(1, n + 1), 2)
        if (min(a, b), max(a, b)) not in seen: seen.add((min(a, b), max(a, b))); print(a, b, r.randint(1, 10**9))
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(2, S + 2); m = r.randint(1, 3 * S); print(n, m)
for _ in range(m):
    a, b = r.sample(range(1, n + 1), 2); print(a, b, r.randint(1, r.choice([5, 10**9])))
