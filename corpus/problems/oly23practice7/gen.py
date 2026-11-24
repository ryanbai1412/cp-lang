"""gen.py SEED SIZE | --max: DMOJ oly23practice7. t test cases, 1 <= n - 1 <= m, sum of m <=
5e5, edges u < v (multi-edges allowed), ranges 1 <= l <= r <= m, edges 1..n-1 a spanning tree.
Each case hides a random weight permutation, takes its MST as the first n - 1 edges and gives
every edge a range around its hidden weight, sometimes shrunk or shifted so the answer is NO.
--max is one case with n = 2e5 on a deep tree and m = 5e5 with wide ranges."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])


def case(n, m, width, deep):
    tree = [(r.randint(max(0, v - 3), v - 1) if deep else r.randint(0, v - 1), v) for v in range(1, n)]
    extra = []
    while len(extra) < m - (n - 1):
        a, b = r.randrange(n), r.randrange(n)
        if a != b:
            extra.append((a, b))
    edges = tree + extra
    w = list(range(1, m + 1))
    r.shuffle(w)
    par = list(range(n))

    def find(x):
        while par[x] != x:
            par[x] = par[par[x]]
            x = par[x]
        return x

    mst, rest = [], []
    for i in sorted(range(m), key=w.__getitem__):
        a, b = find(edges[i][0]), find(edges[i][1])
        (mst if a != b else rest).append(i)
        if a != b:
            par[a] = b
    r.shuffle(rest)
    lines = [f"{n} {m}"]
    for i in mst + rest:
        a, b = sorted(edges[i])
        lo, hi = max(1, w[i] - r.randint(0, width)), min(m, w[i] + r.randint(0, width))
        if not MAX and r.random() < 0.05:
            lo = hi = r.randint(1, m)
        lines.append(f"{a + 1} {b + 1} {lo} {hi}")
    return lines


if MAX:
    cases = [case(200_000, 500_000, 500_000, True)]
else:
    budget = r.randint(1, min(500_000, 6 * S))
    cases = []
    while budget > 0 and len(cases) < 100_000:
        m = r.randint(1, budget)
        n = r.randint(2, m + 1)
        cases.append(case(n, m, r.choice([0, 1, 3, m]), r.random() < 0.5))
        budget -= m
print(len(cases))
for c in cases:
    print("\n".join(c))
