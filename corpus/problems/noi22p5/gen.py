import sys, random
# seed size: T <= 3 cases of n, m <= size + 1 (V small or up to 1e9; sometimes L = R or disjoint
# ranges), often consistent (conditions read off a hidden sequence) so the answer is not -1;
# --max: T = 1, n = m = 1e6 conditions taken from a random hidden sequence
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
T = 1 if mx else r.randint(1, 3)
out = [str(T)]
for _ in range(T):
    n = 10**6 if mx else r.randint(1, min(10**6, S + 1))
    m = 10**6 if mx else r.randint(1, min(10**6, S + 1))
    cap = 10**9 if mx else r.choice([1, 4, 10**9])
    kind = "any" if mx else r.choice(["any", "point", "disjoint"])
    hidden = [r.randint(0, cap) for _ in range(n)] if mx or r.random() < 0.7 else None
    mn = None
    if hidden and n > 1000:
        import math
        lg = [0] * (n + 1)
        for i in range(2, n + 1): lg[i] = lg[i // 2] + 1
        mn = [hidden[:]]
        for j in range(1, lg[n] + 1):
            p, h = mn[-1], 1 << (j - 1)
            mn.append([min(p[i], p[i + h]) for i in range(n - (1 << j) + 1)])
    def rmin(a, b):
        if mn is None: return min(hidden[a:b + 1])
        j = lg[b - a + 1]; return min(mn[j][a], mn[j][b - (1 << j) + 1])
    out.append(f"{n} {m}")
    cuts = sorted(r.sample(range(1, n + 1), min(n, m))) if kind == "disjoint" else None
    for i in range(m):
        if kind == "point": a = b = r.randint(1, n)
        elif kind == "disjoint" and i < len(cuts): a = (cuts[i - 1] + 1) if i else 1; b = cuts[i]; a = min(a, b)
        else:
            w = r.choice([1, 2, 10, n]) if mx else n
            a = r.randint(1, n); b = min(n, a + r.randint(0, w - 1))
        v = rmin(a - 1, b - 1) if hidden else r.randint(0, cap)
        out.append(f"{a} {b} {v}")
print("\n".join(out))
