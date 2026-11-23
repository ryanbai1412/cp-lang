#!/usr/bin/env python3
"""Rebuild-21 differential stress: random small inputs, compared between `cpl run`
(interpreter), the `cpl build` C++ (g++ -O2) and an independent Python brute force.
usage: stress.py PROBLEM N CPL CPPDIR   (run from rebuild-21/)"""
import itertools, math, random, subprocess, sys
from collections import deque
from fractions import Fraction as Fr
from functools import lru_cache

def lines(*rows): return "\n".join(" ".join(map(str, r)) if isinstance(r, (list, tuple)) else str(r) for r in rows) + "\n"

# ---- 01 1974E (and -rev with a purchase cap q) ----
def gen01(rng, rev=False):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        m = rng.randint(1, 8); x = rng.randint(1, 6)
        out.append([m, x, rng.randint(0, 4)] if rev else [m, x])
        for _ in range(m): out.append([rng.choice([0, rng.randint(0, 15)]), rng.randint(1, 5)])
    return lines(*out)
def brute01(inp, rev=False):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        m, x = int(next(tok)), int(next(tok)); q = int(next(tok)) if rev else m
        o = [(int(next(tok)), int(next(tok))) for _ in range(m)]
        best = 0
        for mask in range(1 << m):
            if bin(mask).count("1") > q: continue
            spent, ok, h = 0, True, 0
            for i in range(m):
                if mask >> i & 1:
                    spent += o[i][0]; h += o[i][1]
                    if spent > x * i: ok = False
            if ok: best = max(best, h)
        res.append(best)
    return lines(*res)

# ---- 02 1969C (and -rev with directional budgets) ----
def gen02(rng, rev=False):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        n = rng.randint(1, 5)
        out.append([n, rng.randint(0, 2), rng.randint(0, 2)] if rev else [n, rng.randint(0, 3)])
        out.append([rng.randint(1, 9) for _ in range(n)])
    return lines(*out)
def brute02(inp, rev=False):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n = int(next(tok))
        kl, kr = (int(next(tok)), int(next(tok))) if rev else (int(next(tok)), None)
        a = tuple(int(next(tok)) for _ in range(n))
        start = (a, kl, kr) if rev else (a, kl)
        seen = {start}; dq = deque([start]); best = sum(a)
        while dq:
            st = dq.popleft(); arr = st[0]; best = min(best, sum(arr))
            for i in range(n):
                for d in (-1, 1):
                    j = i + d
                    if not 0 <= j < n: continue
                    b = list(arr); b[i] = arr[j]; b = tuple(b)
                    if rev:
                        l, r = st[1], st[2]
                        if d == -1:
                            if l == 0: continue
                            nx = (b, l - 1, r)
                        else:
                            if r == 0: continue
                            nx = (b, l, r - 1)
                    else:
                        if st[1] == 0: continue
                        nx = (b, st[1] - 1)
                    if nx not in seen: seen.add(nx); dq.append(nx)
        res.append(best)
    return lines(*res)

# ---- 03 1968G1 (and -rev printing f_l..f_r) ----
def gen03(rng, rev=False):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        n = rng.randint(1, 9); l = rng.randint(1, n); r = rng.randint(l, n) if rev else l
        out.append([n, l, r]); out.append("".join(rng.choice("ab" if rng.random() < .7 else "abc") for _ in range(n)))
    return lines(*out)
def brute03(inp, rev=False):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n, l, r = int(next(tok)), int(next(tok)), int(next(tok)); s = next(tok)
        ans = []
        for k in range(l, r + 1):
            best = 0
            for cuts in itertools.combinations(range(1, n), k - 1):
                b = (0,) + cuts + (n,); parts = [s[b[i]:b[i + 1]] for i in range(k)]
                lcp = 0
                while all(len(p) > lcp and p[lcp] == parts[0][lcp] for p in parts): lcp += 1
                best = max(best, lcp)
            ans.append(best)
        res.append(ans if rev else ans[0])
    return lines(*res)

# ---- 04 1954E ----
def gen04(rng):
    n = rng.randint(1, 5); return lines(n, [rng.randint(1, 6) for _ in range(n)])
def brute04(inp):
    v = list(map(int, inp.split())); n, a = v[0], tuple(v[1:])
    out = []
    for k in range(1, max(a) + 1):
        dist = {a: 0}; dq = deque([a])
        while dq:
            st = dq.popleft()
            if not any(st): out.append(dist[st]); break
            i = 0
            while i < n:
                if st[i] == 0: i += 1; continue
                j = i
                while j < n and st[j] > 0: j += 1
                nx = tuple(max(0, st[p] - k) if i <= p < j else st[p] for p in range(n))
                if nx not in dist: dist[nx] = dist[st] + 1; dq.append(nx)
                i = j
    return lines(out)

# ---- 05 1976D ----
def gen05(rng):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        h = rng.randint(1, 6); s = []; bal = 0; o = h
        while o or bal:
            if o and (bal == 0 or rng.random() < .5): s.append("("); o -= 1; bal += 1
            else: s.append(")"); bal -= 1
        out.append("".join(s))
    return lines(*out)
def brute05(inp):
    v = inp.split(); res = []
    def reg(t):
        b = 0
        for c in t:
            b += 1 if c == "(" else -1
            if b < 0: return False
        return b == 0
    for s in v[1:]:
        n = len(s); cnt = 0
        for l in range(n):
            for r in range(l, n):
                t = s[:l] + "".join(")" if c == "(" else "(" for c in s[l:r + 1]) + s[r + 1:]
                cnt += reg(t)
        res.append(cnt)
    return lines(*res)

# ---- 06 1955G ----
def gen06(rng):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        n, m = rng.randint(1, 4), rng.randint(1, 4); out.append([n, m])
        base = rng.choice([1, 2, 3, 4, 6, 12])
        for _ in range(n): out.append([base * rng.randint(1, 12) if rng.random() < .8 else rng.randint(1, 60) for _ in range(m)])
    return lines(*out)
def brute06(inp):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n, m = int(next(tok)), int(next(tok)); a = [[int(next(tok)) for _ in range(m)] for _ in range(n)]
        best = 0
        for moves in set(itertools.permutations("D" * (n - 1) + "R" * (m - 1))):
            r = c = 0; g = a[0][0]
            for mv in moves:
                if mv == "D": r += 1
                else: c += 1
                g = math.gcd(g, a[r][c])
            best = max(best, g)
        res.append(best)
    return lines(*res)

# ---- 07 1978E ----
def gen07(rng):
    T = rng.randint(1, 2); out = [T]
    for _ in range(T):
        n = rng.randint(1, 9); out.append(n)
        out.append("".join(rng.choice("0001") for _ in range(n))); out.append("".join(rng.choice("0111") for _ in range(n)))
        q = rng.randint(1, 6); out.append(q)
        for _ in range(q):
            l = rng.randint(1, n); out.append([l, rng.randint(l, n)])
    return lines(*out)
def brute07(inp):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n = int(next(tok)); s = next(tok); t = next(tok)
        for _ in range(int(next(tok))):
            l, r = int(next(tok)) - 1, int(next(tok))
            a, b = list(s[l:r]), list(t[l:r]); k = r - l; ch = True
            while ch:
                ch = False
                for i in range(k - 2):
                    if a[i] == a[i + 2] == "0" and b[i + 1] == "0": b[i + 1] = "1"; ch = True
                    if b[i] == b[i + 2] == "1" and a[i + 1] == "0": a[i + 1] = "1"; ch = True
            res.append(a.count("1"))
    return lines(*res)

# ---- 08 1957C ----
def gen08(rng):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        n = rng.randint(1, 6); rows = set(); moves = []
        for _ in range(rng.randint(0, n)):
            free = [(r, c) for r in range(1, n + 1) for c in range(1, n + 1) if r not in rows and c not in rows and (r == c or (c not in rows and r not in rows))]
            free = [(r, c) for r, c in free if r not in rows and c not in rows]
            if not free: break
            r, c = rng.choice(free); rows |= {r, c}; moves.append([r, c])
        out.append([n, len(moves)]); out += moves
    return lines(*out)
def brute08(inp):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n, k = int(next(tok)), int(next(tok)); board = set(); used_r = set(); used_c = set()
        for _ in range(k):
            r, c = int(next(tok)), int(next(tok)); board.add((r, c, "W"))
            if r != c: board.add((c, r, "B"))
        finals = set(); seen = set()
        def dfs(bd):
            if bd in seen: return
            seen.add(bd)
            ur = {p[0] for p in bd}; uc = {p[1] for p in bd}; moved = False
            for r in range(1, n + 1):
                for c in range(1, n + 1):
                    if r in ur or c in uc: continue
                    nb = set(bd) | {(r, c, "W")}
                    if r != c: nb.add((c, r, "B"))
                    moved = True; dfs(frozenset(nb))
            if not moved: finals.add(bd)
        dfs(frozenset(board)); res.append(len(finals) % 1_000_000_007)
    return lines(*res)

# ---- 09 1730B ----
def gen09(rng):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        n = rng.randint(1, 5); out.append(n); out.append([rng.randint(0, 10) for _ in range(n)]); out.append([rng.randint(0, 10) for _ in range(n)])
    return lines(*out)
def brute09(inp):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        n = int(next(tok)); x = [int(next(tok)) for _ in range(n)]; t = [int(next(tok)) for _ in range(n)]
        best = min(range(0, 41), key=lambda h: max(2 * t[i] + abs(2 * x[i] - h) for i in range(n)))
        res.append(best / 2)
    return lines(*res)

# ---- 10 1543C ----
def gen10(rng):
    T = rng.randint(1, 3); out = [T]
    for _ in range(T):
        c = rng.randint(1, 9998); m = rng.randint(1, 9999 - c); p = 10000 - c - m; v = rng.randint(1000, 9000)
        out.append(" ".join(f"{z / 10000:.4f}" for z in (c, m, p, v)))
    return lines(*out)
def brute10(inp):
    tok = iter(inp.split()); res = []
    for _ in range(int(next(tok))):
        c, m, p, v = (Fr(next(tok)) for _ in range(4))
        @lru_cache(None)
        def go(st):  # st: probabilities of (cash, marker, pink), 0 = invalid
            e = Fr(1)
            for i in (0, 1):
                a = st[i]
                if a == 0: continue
                nxt = list(st); give = min(a, v); nxt[i] = a - give
                others = [j for j in range(3) if j != i and st[j] > 0]
                for j in others: nxt[j] += give / len(others)
                e += a * go(tuple(nxt))
            return e
        res.append(float(go((c, m, p))))
    return lines(*res)

def same(a, b, tol):
    x, y = a.split(), b.split()
    if len(x) != len(y): return False
    if not tol: return x == y
    try: return all(abs(float(p) - float(q)) <= 1e-6 * max(1, abs(float(q))) for p, q in zip(x, y))
    except ValueError: return False

PROBLEMS = {
    "01-1974E": (gen01, brute01, False), "01-1974E-rev": (lambda r: gen01(r, True), lambda s: brute01(s, True), False),
    "02-1969C": (gen02, brute02, False), "02-1969C-rev": (lambda r: gen02(r, True), lambda s: brute02(s, True), False),
    "03-1968G1": (gen03, brute03, False), "03-1968G1-rev": (lambda r: gen03(r, True), lambda s: brute03(s, True), False),
    "04-1954E": (gen04, brute04, False), "05-1976D": (gen05, brute05, False), "06-1955G": (gen06, brute06, False),
    "07-1978E": (gen07, brute07, False), "08-1957C": (gen08, brute08, False),
    "09-1730B": (gen09, brute09, True), "10-1543C": (gen10, brute10, True),
}

def run(cmd, inp):
    p = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=60)
    return p.returncode, p.stdout, p.stderr

if __name__ == "__main__":
    name, N, cpl, cppdir = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
    gen, brute, tol = PROBLEMS[name]; bad = 0
    for seed in range(N):
        inp = gen(random.Random(seed)); want = brute(inp)
        ic, iout, ierr = run([cpl, "run", f"{name}.cp"], inp)
        cc, cout, cerr = run([f"{cppdir}/{name}"], inp)
        ok_i, ok_c = ic == 0 and same(iout, want, tol), cc == 0 and same(cout, want, tol)
        if not (ok_i and ok_c):
            bad += 1
            if bad <= 3:
                print(f"MISMATCH seed={seed} interp={'ok' if ok_i else 'BAD'} cpp={'ok' if ok_c else 'BAD'}\n--input--\n{inp}--brute--\n{want}--interp(rc={ic})--\n{iout}{ierr[:400]}--cpp(rc={cc})--\n{cout}{cerr[:400]}")
    print(f"{name}: {N - bad}/{N} agree")
