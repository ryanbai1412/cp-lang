import sys, random
if sys.argv[1] == "--max":  # one case n = m = 2e5 (both sums), distinct random segments
    rng = random.Random(0); n = m = 200000; seg = set()
    while len(seg) < m:
        l = rng.randint(1, n); seg.add((l, rng.randint(l, min(n, l + rng.choice([5, 500, n])))))
    print(1); print(n, m); print("\n".join(f"{l} {r}" for l, r in seg)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S); m = r.randint(0, 2 * S); print(n, m)
    for _ in range(m):
        l = r.randint(1, n); print(l, r.randint(l, n))
