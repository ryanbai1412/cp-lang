import sys, random

def case(r, n, m, k, hi):
    cells = set()
    while len(cells) < k:
        cells.add((r.randint(1, n), r.randint(1, m)))
    rows = [r.randint(0, hi) for _ in range(n + 1)]
    cols = [r.randint(0, hi) for _ in range(m + 1)]
    out = [f"{n} {m} {k} 0"]
    for a, b in cells:
        v = rows[a] ^ cols[b]
        if r.random() < 0.05:
            v = r.randint(0, hi)
        out.append(f"{a} {b} {v}")
    return out

if sys.argv[1] == "--max":
    r = random.Random(1)
    lines = case(r, 50000, 50000, 100000, 2**30 - 1)
    lines = [l if i == 0 or r.random() > 0.0 else l for i, l in enumerate(lines)]
    print(1)
    print("\n".join(lines))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 5)
print(t)
for _ in range(t):
    n, m = r.randint(2, max(2, S // 4)), r.randint(2, max(2, S // 4))
    k = r.randint(0, min(n * m, S))
    print("\n".join(case(r, n, m, k, r.choice([1, 7, 2**30 - 1]))))
