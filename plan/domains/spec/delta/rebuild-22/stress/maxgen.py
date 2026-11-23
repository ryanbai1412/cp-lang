#!/usr/bin/env python3
"""One maximum-size input per problem: maxgen.py ID > file"""
import sys, random
R = random.Random(22); pid = sys.argv[1]; o = []; w = o.append
def tree(n):
    return [(R.randint(max(1, v - 3), v - 1) if R.random() < .5 else R.randint(1, v - 1), v) for v in range(2, n + 1)]
if pid == "1988D":
    n = 300000; w("1"); w(str(n)); w(" ".join(str(R.randint(10**12 - 1000, 10**12)) for _ in range(n)))
    o += [f"{p} {v}" for p, v in tree(n)]
elif pid == "2006B":
    n = 200000; W = 10**12; w("1"); w(f"{n} {W}"); st = [1]; par = []
    for i in range(2, n + 1):
        while len(st) > 1 and R.random() < .5: st.pop()
        par.append(st[-1]); st.append(i)
    w(" ".join(map(str, par)))
    cuts = sorted(R.randint(0, W) for _ in range(n - 2)); ys = [b - a for a, b in zip([0] + cuts, cuts + [W])]
    xs = list(range(2, n + 1)); R.shuffle(xs); o += [f"{x} {y}" for x, y in zip(xs, ys)]
elif pid == "2000G":
    n = m = 100000; w("1"); w(f"{n} {m}"); w("1000000000 300000000 600000000")
    es = tree(n) + [tuple(R.sample(range(1, n + 1), 2)) for _ in range(m - n + 1)]
    for u, v in es:
        l1 = R.randint(1, 10**6); w(f"{u} {v} {l1} {R.randint(l1 + 1, 10**7)}")
elif pid == "1998D":
    n = m = 200000; w("1"); w(f"{n} {m}"); s = set()
    while len(s) < m:
        u = R.randint(1, n - 2); v = R.randint(u + 2, min(n, u + R.choice([5, 1000, n])))
        s.add((u, v))
    o += [f"{u} {v}" for u, v in s]
elif pid == "2001D":
    n = 300000; w("1"); w(str(n)); w(" ".join(str(R.randint(1, n)) for _ in range(n)))
elif pid == "2009G1":
    n = q = 200000; k = 1000; w("1"); w(f"{n} {k} {q}"); w(" ".join(str(R.randint(1, n)) for _ in range(n)))
    for _ in range(q): l = R.randint(1, n - k + 1); w(f"{l} {l + k - 1}")
elif pid == "1995B2":
    n = 200000; w("1"); w(f"{n} {10**18}"); a = R.sample(range(1, 10**9), n - 1000)
    a += [x + 1 for x in a[:1000]]; a = list(dict.fromkeys(a))[:n]
    while len(a) < n: a.append(R.randint(1, 10**9)); a = list(dict.fromkeys(a))
    w(" ".join(map(str, a))); w(" ".join(str(R.randint(10**9 - 10, 10**9)) for _ in range(n)))
elif pid == "1994E":
    n = 1000000; w("1"); w("1"); w(str(n)); w(" ".join(str(p) for p, v in tree(n)))
elif pid == "1991D":
    w("1"); w("200000")
elif pid == "1986F":
    n = m = 100000; w("1"); w(f"{n} {m}"); es = set((min(p, v), max(p, v)) for p, v in tree(n))
    while len(es) < m:
        u, v = R.sample(range(1, n + 1), 2); es.add((min(u, v), max(u, v)))
    o += [f"{u} {v}" for u, v in es]
print("\n".join(o))
