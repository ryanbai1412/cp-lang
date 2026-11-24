import sys, random
if sys.argv[1] == "--max":
    # sum q = 1e5 split as one n = 1e9 case with 5e4 queries plus 5e4 one-query cases (per-case
    # cost); |A|, |B|, |C| and t up to 1e9; half the lines are diagonals (+-1, +-1) or axis lines
    # through diamond sides, so the parallel interval-union path runs.
    r = random.Random(0); n = 10**9; L = 10**9
    def query(n, x, y):
        t = r.choice([L, r.randint(0, L), r.randint(0, 2 * n)])
        if r.random() < 0.5:
            a, b = r.choice([(1, 1), (1, -1), (-1, 1), (0, 1), (1, 0), (L, -L), (L, L)])
            k = L // max(abs(a), abs(b))
            c = max(-L, min(L, -(a * x + b * y) + (a * r.choice([-1, 1]) * min(t, k) if r.random() < 0.7 else 0)))
        else:
            a, b, c = r.randint(-L, L), r.randint(-L, L), r.randint(-L, L)
            if a == 0 and b == 0: a = 1
        return f"{a} {b} {c} {t}"
    out = [str(50001)]
    x, y = r.randint(1, n), r.randint(1, n)
    out.append(f"{n} {x} {y} 50000")
    out += [query(n, x, y) for _ in range(50000)]
    for _ in range(50000):
        m = r.choice([n, r.randint(2, n), 2])
        x, y = r.randint(1, m), r.randint(1, m)
        out.append(f"{m} {x} {y} 1"); out.append(query(m, x, y))
    print("\n".join(out)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    big = r.random() < 0.2
    n = r.randint(2, 10**9 if big else 2 + 2 * S); q = 6
    print(n, r.randint(1, n), r.randint(1, n), q)
    for _ in range(q):
        k = 1 if big else n   # keep c = -(a*x + b*y) inside 32 bits
        a, b = r.randint(-k, k), r.randint(-k, k)
        if a == 0 and b == 0: a = 1
        if r.random() < 0.5: a, b = r.choice([(1, 1), (1, -1), (-1, 1), (0, 1), (1, 0), (2, -2), (a, b)])
        x, y = r.randint(1, n), r.randint(1, n)
        c = -(a * x + b * y) if r.random() < 0.8 else r.randint(-n, n)
        print(a, b, c, r.randint(0, 10**9 if big else 5 * n))
