import random, itertools, sys
from fractions import Fraction
from collections import deque
MOD = 10**9+7

def multi(gen_one, solve_one, rng, tmax=3):
    t = rng.randint(1, tmax)
    cases = [gen_one(rng) for _ in range(t)]
    inp = f"{t}\n" + "".join(c[0] for c in cases)
    out = "".join(solve_one(c[1]) + "\n" for c in cases)
    return inp, out

# 01 1918D
def g1918(r):
    n = r.randint(1, 7); a = [r.randint(1, 10) for _ in range(n)]
    return f"{n}\n{' '.join(map(str,a))}\n", a
def s1918(a):
    n = len(a); best = None
    for mask in range(1 << n):
        blocked = sum(a[i] for i in range(n) if mask >> i & 1)
        seg = cur = 0
        for i in range(n):
            if mask >> i & 1: cur = 0
            else: cur += a[i]; seg = max(seg, cur)
        c = max(blocked, seg); best = c if best is None else min(best, c)
    return str(best)

# 02 1905D
def g1905(r):
    n = r.randint(1, 7); p = list(range(n)); r.shuffle(p)
    return f"{n}\n{' '.join(map(str,p))}\n", p
def s1905(p):
    n = len(p); best = 0
    for k in range(n):
        q = p[k:] + p[:k]; tot = 0
        for L in range(1, n + 1):
            st = set(q[:L]); m = 0
            while m in st: m += 1
            tot += m
        best = max(best, tot)
    return str(best)

# 03 1915G (state Bellman-Ford over (city, slowness value))
def g1915(r):
    n = r.randint(2, 6); m = r.randint(n - 1, 8); es = []
    for v in range(2, n + 1): es.append((r.randint(1, v - 1), v, r.randint(1, 20)))
    while len(es) < m:
        u = r.randint(1, n); v = r.randint(1, n)
        if u != v: es.append((u, v, r.randint(1, 20)))
    r.shuffle(es); s = [r.randint(1, 10) for _ in range(n)]
    txt = f"{n} {len(es)}\n" + "".join(f"{u} {v} {w}\n" for u, v, w in es) + " ".join(map(str, s)) + "\n"
    return txt, (n, es, s)
def all1915(c):
    n, es, s = c; INF = float('inf')
    d = {(1, s[0]): 0}; changed = True
    while changed:
        changed = False
        for (v, sl), dv in list(d.items()):
            for u, w, x in es:
                for a, b in ((u, w), (w, u)):
                    if a != v: continue
                    ns = min(sl, s[b - 1]); nd = dv + x * sl
                    if nd < d.get((b, ns), INF): d[(b, ns)] = nd; changed = True
    return [min(dv for (v, _), dv in d.items() if v == c) for c in range(1, n + 1)]
def s1915(c): return str(all1915(c)[-1])
def s1915rev(c): return " ".join(map(str, all1915(c)[1:]))

# 04 1924B (single test)
def gen1924(r):
    n = r.randint(2, 9); hs = {1, n}
    for x in range(2, n):
        if r.random() < 0.3: hs.add(x)
    hs = sorted(hs); r.shuffle(hs); val = {x: r.randint(1, 20) for x in hs}
    q = r.randint(1, 8); qs = []; cur = dict(val); out = []
    def cost(i):
        if i in cur: return 0
        L = max(h for h in cur if h < i); R = min(h for h in cur if h > i)
        return cur[L] * (R - i)
    for _ in range(q):
        free = [x for x in range(2, n) if x not in cur]
        if free and r.random() < 0.4:
            x = r.choice(free); v = r.randint(1, 20); cur[x] = v; qs.append(f"1 {x} {v}")
        else:
            l = r.randint(1, n); rr = r.randint(l, n); qs.append(f"2 {l} {rr}")
            out.append(str(sum(cost(i) for i in range(l, rr + 1))))
    if not out:
        qs.append(f"2 1 {n}"); out.append(str(sum(cost(i) for i in range(1, n + 1))))
    inp = f"{n} {len(hs)} {len(qs)}\n{' '.join(map(str,hs))}\n{' '.join(str(val[x]) for x in hs)}\n" + "\n".join(qs) + "\n"
    return inp, "\n".join(out) + "\n"

# 05 1930D2
def g1930(r):
    n = r.randint(1, 7); s = "".join(r.choice("01") for _ in range(n))
    return f"{n}\n{s}\n", s
FC = {}
def f1930(p):
    if p in FC: return FC[p]
    m = len(p); best = m
    for q in itertools.product((0, 1), repeat=m):
        if sum(q) >= best: continue
        ok = True
        for i in range(m):
            want = int(p[i]); found = False
            for l in range(i + 1):
                for rr in range(i, m):
                    ones = sum(q[l:rr + 1]); zeros = rr + 1 - l - ones
                    if (want == 1 and ones >= zeros) or (want == 0 and zeros >= ones): found = True; break
                if found: break
            if not found: ok = False; break
        if ok: best = sum(q)
    FC[p] = best; return best
def s1930(s):
    return str(sum(f1930(s[i:j]) for i in range(len(s)) for j in range(i + 1, len(s) + 1)))

# 07 1929E
def g1929(r, cost=False):
    n = r.randint(2, 8); es = []
    for v in range(2, n + 1): es.append((r.randint(1, v - 1), v, r.randint(1, 9)))
    k = r.randint(1, 4); ps = []
    for _ in range(k):
        a, b = r.sample(range(1, n + 1), 2); ps.append((a, b))
    txt = f"{n}\n" + "".join(f"{u} {v} {c}\n" if cost else f"{u} {v}\n" for u, v, c in es) + f"{k}\n" + "".join(f"{a} {b}\n" for a, b in ps)
    return txt, (n, es, ps, cost)
def s1929(c):
    n, es, ps, cost = c
    adj = {v: [] for v in range(1, n + 1)}
    for i, (u, v, _) in enumerate(es): adj[u].append((v, i)); adj[v].append((u, i))
    def path(a, b):
        st = [(a, -1, [])]
        while st:
            v, p, e = st.pop()
            if v == b: return set(e)
            for u, i in adj[v]:
                if u != p: st.append((u, v, e + [i]))
    paths = [path(a, b) for a, b in ps]; best = None
    for mask in range(1 << len(es)):
        if all(any(mask >> i & 1 for i in P) for P in paths):
            c2 = sum(es[i][2] if cost else 1 for i in range(len(es)) if mask >> i & 1)
            best = c2 if best is None else min(best, c2)
    return str(best)

# 08 1925D
def g1925(r):
    n = r.randint(2, 4); allp = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    m = r.randint(0, len(allp)); fr = r.sample(allp, m); k = r.randint(1, 4)
    fr = [(a, b, r.randint(1, 10)) if r.random() < .5 else (b, a, r.randint(1, 10)) for a, b in fr]
    return f"{n} {m} {k}\n" + "".join(f"{a} {b} {f}\n" for a, b, f in fr), (n, fr, k)
def s1925(c):
    n, fr, k = c; pairs = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    val = {(min(a, b), max(a, b)): f for a, b, f in fr}; tot = Fraction(0)
    for seq in itertools.product(range(len(pairs)), repeat=k):
        cur = dict(val); s = 0
        for i in seq:
            pr = pairs[i]
            if pr in cur: s += cur[pr]; cur[pr] += 1
        tot += s
    e = tot / len(pairs) ** k
    return str(e.numerator % MOD * pow(e.denominator, MOD - 2, MOD) % MOD)

# 09 1948D
def g1948(r):
    n = r.randint(1, 10); s = "".join(r.choice("ab?") for _ in range(n)); return s + "\n", s
def s1948(s):
    best = 0
    for i in range(len(s)):
        for L in range(2, len(s) - i + 1, 2):
            h = L // 2
            if all(s[i + j] == s[i + h + j] or '?' in (s[i + j], s[i + h + j]) for j in range(h)): best = max(best, L)
    return str(best)

# 10 1923D
def g1923(r):
    n = r.randint(1, 6); a = [r.randint(1, 4) for _ in range(n)]
    return f"{n}\n{' '.join(map(str,a))}\n", a
def s1923(a):
    n = len(a); res = []
    for t in range(n):
        start = tuple((a[i], i == t) for i in range(n)); seen = {start}; dq = deque([(start, 0)]); ans = -1
        while dq:
            st, d = dq.popleft()
            if not any(f for _, f in st): ans = d; break
            for i in range(len(st) - 1):
                for x, y in ((i, i + 1), (i + 1, i)):
                    if st[x][0] > st[y][0]:
                        lst = list(st); lst[x] = (st[x][0] + st[y][0], st[x][1]); del lst[y]; ns = tuple(lst)
                        if ns not in seen: seen.add(ns); dq.append((ns, d + 1))
        res.append(ans)
    return " ".join(map(str, res))
