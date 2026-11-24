import sys, random
if sys.argv[1] == "--max":  # n = q = 1e6, weights up to 1e9, runs of each direction, wide queries
    rng = random.Random(0); n = q = 10**6; out = [str(n)]; d = "L"
    for _ in range(n):
        if rng.random() < 0.3: d = rng.choice("LR")
        out.append(f"{rng.choice([rng.randint(1, 10**9), 10**9])} {d}")
    out.append(str(q))
    for _ in range(q):
        l = rng.randint(1, n // 10); out.append(f"{l} {rng.randint(n - n // 10, n)}")
    print("\n".join(out)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, 3 * S + 4); print(n)
for _ in range(n):
    print(r.randint(1, 10**9), r.choice("LR"))
q = r.randint(1, 2 * S + 4); print(q)
for _ in range(q):
    l = r.randint(1, n); ri = r.randint(l, n)
    print(l, ri)
