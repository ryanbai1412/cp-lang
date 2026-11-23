#!/usr/bin/env python3
"""Generators and brutes for rebuild round 22. Usage: stress.py gen ID SEED | brute ID | valid ID IN OUT"""
import sys, random, itertools
from collections import deque, Counter

def toks():
    return iter(sys.stdin.read().split())

# ---------- generators (one test per input, t = 1) ----------
def rand_tree(n, R):
    return [(R.randint(1, v - 1), v) for v in range(2, n + 1)]

def gen(pid, seed):
    R = random.Random(seed); o = ["1"]
    if pid == "1988D":
        n = R.randint(1, 6); o.append(str(n))
        big = R.random() < 0.5
        o.append(" ".join(str(R.randint(1, 10**12 if big else 20)) for _ in range(n)))
        perm = list(range(1, n + 1)); R.shuffle(perm)
        for p, v in rand_tree(n, R): o.append(f"{perm[p-1]} {perm[v-1]}")
    elif pid == "2006B":
        n = R.randint(2, 8); w = R.choice([R.randint(0, 20), R.randint(0, 10**12)])
        ch = {1: []}
        for v in range(2, n + 1): p = R.randint(1, v - 1); ch[v] = []; ch[p].append(v)
        lab = {}; st = [1]
        while st:
            u = st.pop(); lab[u] = len(lab) + 1; st.extend(reversed(ch[u]))
        par = {}
        for u in ch:
            for c in ch[u]: par[lab[c]] = lab[u]
        o.append(f"{n} {w}"); o.append(" ".join(str(par[i]) for i in range(2, n + 1)))
        cuts = sorted(R.randint(0, w) for _ in range(n - 2)); ys = [b - a for a, b in zip([0] + cuts, cuts + [w])]
        xs = list(range(2, n + 1)); R.shuffle(xs)
        for x, y in zip(xs, ys): o.append(f"{x} {y}")
    elif pid == "2001D":
        n = R.randint(1, 8); o += [str(n), " ".join(str(R.randint(1, n)) for _ in range(n))]
    elif pid == "2009G1":
        n = R.randint(1, 10); k = R.randint(1, n); q = R.randint(1, 5)
        o += [f"{n} {k} {q}", " ".join(str(R.randint(1, n)) for _ in range(n))]
        for _ in range(q): l = R.randint(1, n - k + 1); o.append(f"{l} {l + k - 1}")
    elif pid == "1995B2":
        n = R.randint(1, 4); a = R.sample(range(1, 9), n); m = R.randint(1, 60)
        o += [f"{n} {m}", " ".join(map(str, a)), " ".join(str(R.randint(1, 5)) for _ in range(n))]
    elif pid == "1986F":
        n = R.randint(2, 7); es = set((min(p, v), max(p, v)) for p, v in rand_tree(n, R))
        for _ in range(R.randint(0, 4)):
            u, v = R.sample(range(1, n + 1), 2); es.add((min(u, v), max(u, v)))
        es = list(es); R.shuffle(es)
        o.append(f"{n} {len(es)}"); o += [f"{u} {v}" if R.random() < .5 else f"{v} {u}" for u, v in es]
    elif pid == "1994E":
        k = R.randint(1, 3); o.append(str(k))
        for _ in range(k):
            n = R.randint(1, 7); o.append(str(n)); o.append(" ".join(str(R.randint(1, v - 1)) for v in range(2, n + 1)))
    elif pid == "1991D":
        o.append(str(R.randint(1, 12)))
    return "\n".join(o) + "\n"

# ---------- brutes ----------
def brute(pid):
    it = toks(); t = int(next(it)); out = []
    for _ in range(t):
        if pid == "1988D":
            n = int(next(it)); a = [int(next(it)) for _ in range(n)]
            es = [(int(next(it)) - 1, int(next(it)) - 1) for _ in range(n - 1)]
            best = min(sum(a[i] * r[i] for i in range(n)) for r in itertools.product(range(1, n + 1), repeat=n)
                       if all(r[u] != r[v] for u, v in es))
            out.append(str(best))
        elif pid == "2006B":
            n, w = int(next(it)), int(next(it)); par = {i: int(next(it)) for i in range(2, n + 1)}
            def up(v):
                p = [v]
                while v != 1: v = par[v]; p.append(v)
                return p
            def path(u, v):
                a, b = up(u), up(v); sa = set(a); l = next(x for x in b if x in sa)
                return [x for x in a[:a.index(l)]] + [x for x in b[:b.index(l)]]  # edges named by child
            known = {}; res = []
            for _ in range(n - 1):
                x, y = int(next(it)), int(next(it)); known[x] = y; rest = w - sum(known.values()); s = 0
                for i in range(1, n + 1):
                    e = path(i, i % n + 1); s += sum(known.get(x, 0) for x in e) + (rest if any(x not in known for x in e) else 0)
                res.append(str(s))
            out.append(" ".join(res))
        elif pid == "2001D":
            n = int(next(it)); a = [int(next(it)) for _ in range(n)]; best = None
            for mask in range(1, 1 << n):
                b = [a[i] for i in range(n) if mask >> i & 1]
                if len(set(b)) != len(b): continue
                key = (-len(b), [-x if j % 2 == 0 else x for j, x in enumerate(b)])
                if best is None or key < best[0]: best = (key, b)
            b = best[1]; out += [str(len(b)), " ".join(map(str, b))]
        elif pid == "2009G1":
            n, k, q = (int(next(it)) for _ in range(3)); a = [int(next(it)) for _ in range(n)]
            for _ in range(q):
                l, r = int(next(it)) - 1, int(next(it))
                b = a[l:r]; out.append(str(min(sum(1 for j in range(k) if b[j] != x + j) for x in range(-20, 30))))
        elif pid == "1995B2":
            n, m = int(next(it)), int(next(it)); a = [int(next(it)) for _ in range(n)]; c = [int(next(it)) for _ in range(n)]
            q = dict(zip(a, c)); best = 0
            for x in a:
                for i in range(q[x] + 1):
                    for j in range(q.get(x + 1, 0) + 1):
                        s = i * x + j * (x + 1)
                        if s <= m: best = max(best, s)
            out.append(str(best))
        elif pid == "1986F":
            n, m = int(next(it)), int(next(it)); es = [(int(next(it)), int(next(it))) for _ in range(m)]; best = None
            for skip in range(m):
                adj = {v: [] for v in range(1, n + 1)}
                for i, (u, v) in enumerate(es):
                    if i != skip: adj[u].append(v); adj[v].append(u)
                seen = set(); tot = 0
                for s in adj:
                    if s in seen: continue
                    comp = 0; dq = deque([s]); seen.add(s)
                    while dq:
                        u = dq.popleft(); comp += 1
                        for v in adj[u]:
                            if v not in seen: seen.add(v); dq.append(v)
                    tot += comp * (comp - 1) // 2
                best = tot if best is None else min(best, tot)
            out.append(str(best))
        elif pid == "1994E":
            k = int(next(it)); acc = {0}
            for _ in range(k):
                n = int(next(it)); par = {v: int(next(it)) for v in range(2, n + 1)}
                def sub(v, rem):
                    s = {v}; ch = True
                    while ch:
                        ch = False
                        for x in rem:
                            if x not in s and x in par and par[x] in s: s.add(x); ch = True
                    return s
                ors = set(); seen = set(); st = [(frozenset(range(1, n + 1)), 0)]
                while st:
                    rem, o_ = st.pop()
                    if (rem, o_) in seen: continue
                    seen.add((rem, o_))
                    if not rem: ors.add(o_); continue
                    for v in rem:
                        s = sub(v, rem); st.append((rem - s, o_ | len(s)))
                acc = {x | y for x in acc for y in ors}
            out.append(str(max(acc)))
        elif pid == "1991D":
            n = int(next(it)); out.append(str(chromatic(n)))
    print("\n".join(out))

def isp(x): return x > 1 and all(x % d for d in range(2, int(x ** .5) + 1))
def chromatic(n):
    adj = [[v for v in range(1, n + 1) if isp(u ^ v)] for u in range(n + 1)]
    for k in range(1, n + 1):
        col = [0] * (n + 1)
        def go(u):
            if u > n: return True
            for c in range(1, k + 1):
                if all(col[v] != c for v in adj[u] if v < u):
                    col[u] = c
                    if go(u + 1): return True
            col[u] = 0; return False
        if go(1): return k

def valid(pid, inp, outp):
    """1991D: output is a minimum proper colouring."""
    it = iter(open(inp).read().split()); ot = iter(open(outp).read().split()); t = int(next(it))
    for _ in range(t):
        n = int(next(it)); k = int(next(ot)); c = [0] + [int(next(ot)) for _ in range(n)]
        if k != chromatic(n) or any(not 1 <= x <= k for x in c[1:]): return False
        if any(c[u] == c[v] and isp(u ^ v) for u in range(1, n + 1) for v in range(u + 1, n + 1)): return False
    return True

if __name__ == "__main__":
    if sys.argv[1] == "gen": sys.stdout.write(gen(sys.argv[2], int(sys.argv[3])))
    elif sys.argv[1] == "brute": brute(sys.argv[2])
    else: sys.exit(0 if valid(sys.argv[2], sys.argv[3], sys.argv[4]) else 1)
