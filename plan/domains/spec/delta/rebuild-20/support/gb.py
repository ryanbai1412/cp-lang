import sys, random, math, itertools
from collections import deque
def gen(p, sd):
    r = random.Random(sd); o = []
    if p == '05':
        n = r.randint(1, 5); o.append(n)
        for _ in range(n): o.append(''.join(r.choice('ab') for _ in range(r.randint(1, 4))))
    elif p == '06':
        t = r.randint(1, 3); o.append(t)
        for _ in range(t):
            n = r.randint(1, 6); a = [r.randint(1, 3) for _ in range(n)]
            if r.random() < .5:
                a.sort(reverse=r.random() < .5); k = r.randint(0, n-1); a = a[k:] + a[:k]
            o.append(n); o.append(' '.join(map(str, a)))
    elif p == '07':
        n = r.randint(3, 10); o.append(n); o.append(' '.join(str(r.randint(1, 4)) for _ in range(n)))
    elif p == '08':
        t = r.randint(1, 3); o.append(t)
        for _ in range(t):
            n = r.randint(3, 8); o.append(n); o.append(' '.join(str(r.randint(1, r.choice([6, 30, 100000]))) for _ in range(n)))
    elif p == '09':
        t = r.randint(1, 3); o.append(t)
        for _ in range(t):
            n = r.randint(1, 8); o.append(n); o.append(' '.join(str(r.randint(1, i-1)) for i in range(2, n+1)))
            o.append(' '.join(str(r.randint(1, r.choice([min(2, n), n]))) for _ in range(n)))
    elif p == '10':
        n = r.randint(1, 8); q = r.randint(1, 5); o.append(f"{n} {q}"); o.append(''.join(r.choice('UDLR') for _ in range(n)))
        for _ in range(q):
            l = r.randint(1, n); rr = r.randint(l, n); m = min(n, 3)
            o.append(f"{r.randint(-m, m)} {r.randint(-m, m)} {l} {rr}")
    return '\n'.join(map(str, o)) + '\n'
def brute(p, s):
    it = iter(s.split()); nx = lambda: next(it); out = []
    if p == '05':
        n = int(nx()); ss = [nx() for _ in range(n)]; tot = 0
        for a in ss:
            for b in ss:
                i, j = len(a), 0
                while i > 0 and j < len(b) and a[i-1] == b[j]: i -= 1; j += 1
                tot += i + len(b) - j
        out.append(tot)
    elif p == '06':
        for _ in range(int(nx())):
            n = int(nx()); a = tuple(int(nx()) for _ in range(n)); tgt = tuple(sorted(a))
            d = {a: 0}; dq = deque([a]); ans = -1
            while dq:
                u = dq.popleft()
                if u == tgt: ans = d[u]; break
                for v in (u[-1:] + u[:-1], u[::-1]):
                    if v not in d: d[v] = d[u] + 1; dq.append(v)
            out.append(ans)
    elif p == '07':
        n = int(nx()); a = [int(nx()) for _ in range(n)]; c = 0
        for mask in range(1 << n):
            b = [a[i] for i in range(n) if mask >> i & 1]
            if len(b) >= 3 and all((b[i]+b[i+1]+b[i+2]) % 2 == 0 for i in range(len(b)-2)): c += 1
        out.append(c % 998244353)
    elif p == '08':
        for _ in range(int(nx())):
            n = int(nx()); a = [int(nx()) for _ in range(n)]; s2 = 0
            for i, j, k in itertools.combinations(range(n), 3):
                x = sorted([a[i], a[j], a[k]]); s2 += math.gcd(x[0], x[1])
            out.append(s2)
    elif p == '09':
        for _ in range(int(nx())):
            n = int(nx()); par = [0, 0] + [int(nx()) for _ in range(n-1)]; col = [0] + [int(nx()) for _ in range(n)]
            dep = [0]*(n+1)
            for v in range(2, n+1): dep[v] = dep[par[v]] + 1
            def up(u, l):
                cs = {col[u]}
                while u != l: u = par[u]; cs.add(col[u])
                return len(cs)
            best = 0
            for u in range(1, n+1):
                for v in range(1, n+1):
                    x, y = u, v
                    while x != y:
                        if dep[x] < dep[y]: x, y = y, x
                        x = par[x]
                    best = max(best, up(u, x) * up(v, x))
            out.append(best)
    elif p == '10':
        n, q = int(nx()), int(nx()); s0 = nx()
        for _ in range(q):
            x, y, l, r = (int(nx()) for _ in range(4)); t = s0[:l-1] + s0[l-1:r][::-1] + s0[r:]
            cx = cy = 0; hit = (x, y) == (0, 0)
            for ch in t:
                cx += (ch == 'R') - (ch == 'L'); cy += (ch == 'U') - (ch == 'D'); hit |= (cx, cy) == (x, y)
            out.append("YES" if hit else "NO")
    return '\n'.join(map(str, out)) + '\n'
if __name__ == '__main__':
    if sys.argv[1] == 'gen': sys.stdout.write(gen(sys.argv[2], int(sys.argv[3])))
    else: sys.stdout.write(brute(sys.argv[2], sys.stdin.read()))
