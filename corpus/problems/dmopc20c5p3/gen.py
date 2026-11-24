"""gen.py SEED SIZE | --max: DMOPC '20 Contest 5 P3 (Bottom Row). An N x N grid, 2 <= N <= 1e6,
with K <= N blocked cells on distinct rows and columns, never (1, 1) or (N, N). Blocks are a
random partial permutation, diagonal chains, or anti-diagonal walls (often IMPOSSIBLE).
--max is N = 1e6 with K = N: the super-diagonal (r, r + 1) forces a full zigzag, plus (N, 1)."""
import random, sys
if sys.argv[1] == "--max":
    n = 10**6
    out = [f"{n} {n}"] + [f"{r} {r + 1}" for r in range(1, n)] + [f"{n} 1"]
    sys.stdout.write("\n".join(out) + "\n")
    sys.exit()
r = random.Random(int(sys.argv[1]))
n = r.randint(2, max(2, min(10**6, int(sys.argv[2]) + 1)))
rows, cols = list(range(1, n + 1)), list(range(1, n + 1))
mode = r.randrange(4)
if mode == 0:
    r.shuffle(cols)
    pairs = list(zip(rows, cols))
elif mode == 1:
    s = r.randint(-2, 2)
    pairs = [(i, (i - 1 + s) % n + 1) for i in rows]
elif mode == 2:
    pairs = [(i, n + 1 - i) for i in rows]
    r.shuffle(pairs)
    pairs = pairs[: r.randint(0, n)]
else:
    pairs, used = [], set()
    i = 1
    while i <= n:
        ln = r.randint(1, 4)
        c0 = r.randint(1, n)
        for j in range(ln):
            if i + j <= n and c0 + j <= n and c0 + j not in used:
                pairs.append((i + j, c0 + j))
                used.add(c0 + j)
        i += ln + r.randint(0, 2)
pairs = [p for p in pairs if p not in ((1, 1), (n, n))]
r.shuffle(pairs)
pairs = pairs[: r.randint(0, len(pairs))] if r.random() < 0.5 else pairs
sys.stdout.write("\n".join([f"{n} {len(pairs)}"] + [f"{a} {b}" for a, b in pairs]) + "\n")
