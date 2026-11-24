import sys, random
# --max: T = 3 tests at n = 2e5, m = 4e5, q = 4e5, S = 1e9: a random graph, a path (walks near
# 2e9) and few altitudes.
MAX = sys.argv[1] == "--max"
if MAX:
    sys.argv[1:] = ["0", "1000000"]
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)


def case(n, m, q, s, path, alt_hi):
    edges, seen = [], set()
    order = list(range(1, n + 1))
    if not path:
        rng.shuffle(order)
    for i in range(1, n):
        u, v = order[i], order[i - 1] if path else order[rng.randrange(i)]
        edges.append((u, v))
        seen.add((min(u, v), max(u, v)))
    while len(edges) < m:
        u, v = rng.randint(1, n), rng.randint(1, n)
        if u != v and (min(u, v), max(u, v)) not in seen:
            seen.add((min(u, v), max(u, v)))
            edges.append((u, v))
    rng.shuffle(edges)
    lhi = 10**4 if size > 20 else rng.choice([3, 10**4])
    out = [f"{n} {m}"]
    out += [f"{u} {v} {rng.randint(1, lhi)} {rng.randint(1, alt_hi)}" for u, v in edges]
    k = rng.randint(0, 1)
    out.append(f"{q} {k} {s}")
    out += [f"{rng.randint(1, n)} {rng.randint(0, s)}" for _ in range(q)]
    return out


if size <= 20:
    t = rng.randint(1, 3)
    out = [str(t)]
    for _ in range(t):
        n = rng.randint(1, size + 2)
        m = min(rng.randint(n - 1, n + 4), n * (n - 1) // 2)
        s = rng.choice([1, 5, 10**9])
        out += case(n, m, rng.randint(1, 10), s, rng.random() < 0.3, min(s + 1, 10**9))
else:
    # 0: random graph, all maxima; 1: a path of 1e4 roads, walks near 2e9; 2: few altitudes.
    out = ["3" if MAX else "1"]
    for mode in range(3) if MAX else [seed % 3]:
        n = 200_000
        m = n - 1 if mode == 1 else 400_000
        out += case(n, m, 400_000, 10**9, mode == 1, 5 if mode == 2 else 10**9)
print("\n".join(out))
