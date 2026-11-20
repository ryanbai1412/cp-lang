#!/usr/bin/env python3
"""Differential check for scout batch 45.

Builds every today/<name>.cp with the prototype compiler (on a copy, since
`cp.py build` stamps its input), compiles the result under ASan/UBSan, and
runs it against the original corpus submission on random small and medium
inputs. Where the answer is not unique a validator replaces the token diff,
and where a tiny exhaustive solver exists it is run on the small inputs too.

    python3 proto/ports/scout_batch_45/check.py [name ...] [--cases N]
"""
import itertools, os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "shoi15p2": 5450923, "shoi16p1": 5445691, "shoi16p2": 5445692, "shoi16p5": 5445695,
    "ucrpc21h": 3950286, "iati21a12": 5071843, "iati21a23": 5072008, "neerc10k": 5325567,
    "rpeng2": 4810287, "noimps22p5": 5448138,
}
R = random.Random(45)


def sh(cmd, **kw):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)


def build(name, tmp):
    shutil.copy(f"{HERE}/today/{name}.cp", f"{tmp}/{name}.cp")
    r = sh(f"python3 {ROOT}/proto/cp.py build {tmp}/{name}.cp")
    if not os.path.exists(f"{tmp}/{name}.cpp"):
        sys.exit(f"{name}: prototype build failed\n{r.stdout}{r.stderr}")
    flags = "-std=c++17 -O1 -g -w -fsanitize=address,undefined -fno-sanitize-recover=all"
    r = sh(f"g++ {flags} -o {tmp}/{name}_cp {tmp}/{name}.cpp")
    if r.returncode:
        sys.exit(f"{name}: emitted C++ failed to compile\n{r.stderr[:3000]}")
    r = sh(f"g++ -std=c++17 -O1 -w -o {tmp}/{name}_orig {ROOT}/corpus/submissions/{ORIG[name]}.cpp")
    if r.returncode:
        sys.exit(f"{name}: original failed to compile\n{r.stderr[:3000]}")


def tree(n):
    return [(R.randint(1, i - 1), i) for i in range(2, n + 1)]


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


# ---- generators: gen(big) -> input text ------------------------------------

def gen_shoi15p2(big):
    n = R.randint(1, 60 if big else 8)
    rows = []
    for _ in range(R.randint(1, 80 if big else 10)):
        t = R.choice([0, 1, 2, 2])
        l = R.randint(1, n); r = R.randint(l, n)
        if t == 1:
            l1 = R.randint(1, n); r1 = R.randint(l1, n)
            rows.append((t, l, r, l1, r1))
        else:
            rows.append((t, l, r))
    return lines((n, len(rows)), *rows)


def gen_shoi16p1(big):
    n = R.randint(1, 60 if big else 9)
    k = R.randint(0, 4 if big else 3)
    marked = R.sample(range(1, n + 1), R.randint(0, n))
    es = tree(n)
    R.shuffle(es)
    return lines((n, k), [R.randint(1, 20) for _ in range(n)], (len(marked),), marked, *es)


def brute_shoi16p1(text):
    t = text.split(); n, k = int(t[0]), int(t[1])
    cost = list(map(int, t[2:2 + n])); m = int(t[2 + n])
    marked = [int(x) - 1 for x in t[3 + n:3 + n + m]]
    es = list(map(int, t[3 + n + m:]))
    d = [[0 if i == j else 99 for j in range(n)] for i in range(n)]
    for i in range(0, len(es), 2):
        u, v = es[i] - 1, es[i + 1] - 1
        d[u][v] = d[v][u] = 1
    for c in range(n):
        for a in range(n):
            for b in range(n):
                d[a][b] = min(d[a][b], d[a][c] + d[c][b])
    best = 10 ** 9
    for mask in range(1 << n):
        g = [i for i in range(n) if mask >> i & 1]
        if all(any(d[u][x] <= k for x in g) for u in marked):
            best = min(best, sum(cost[i] for i in g))
    return str(best)


def gen_shoi16p2(big):
    n = R.randint(1, 40 if big else 5); m = R.randint(1, 40 if big else 5)
    pts = [(x, y) for x in range(n + 1) for y in range(m + 1)]
    k = R.randint(0, min(len(pts), 25 if big else 8))
    return lines((n, m, k), *R.sample(pts, k))


def brute_shoi16p2(text):
    t = list(map(int, text.split())); n, m, k = t[:3]
    bad = {(t[3 + 2 * i], t[4 + 2 * i]) for i in range(k)}
    cnt = 0
    for x in range(n + 1):
        for y in range(m + 1):
            for dx in range(1, n + 1):          # each square once: its edge with dx > 0, dy >= 0
                for dy in range(0, m + 1):
                    sq = [(x, y), (x + dx, y + dy), (x + dx - dy, y + dy + dx), (x - dy, y + dx)]
                    if all(0 <= a <= n and 0 <= b <= m for a, b in sq) and not any(p in bad for p in sq):
                        cnt += 1
    return str(cnt % 100000007)


def gen_shoi16p5(big):
    n = R.randint(1, 50 if big else 6)
    hi = 10 ** 9 if big else 9
    rows = [(R.randint(1, n), R.randint(0, hi)) for _ in range(R.randint(1, 30 if big else 5))]
    return lines((n, len(rows)), [R.randint(0, hi) for _ in range(n)], *rows)


def brute_shoi16p5(text):
    t = list(map(int, text.split())); n, q = t[:2]
    a = t[2:2 + n]; out = []
    for i in range(q):
        a[t[2 + n + 2 * i] - 1] = t[3 + n + 2 * i]
        tot = 0
        for ops in itertools.product("+-*", repeat=n - 1):
            tot += eval("".join(str(v) + o for v, o in zip(a, ops + ("",))))
        out.append(str(tot % 1000000007))
    return " ".join(out)


def gen_ucrpc21h(big):
    n = R.randint(1, 12 if big else 6)
    hi = R.choice([5, 40, 120, 320]) if big else R.choice([3, 10, 60, 310])
    es = [(u - 1, v - 1, R.randint(1, hi)) for u, v in tree(n)]
    for _ in range(R.randint(0, n)):
        es.append((R.randrange(n), R.randrange(n), R.randint(1, hi)))
    R.shuffle(es)
    return lines((n, len(es)), *es)


def brute_ucrpc21h(text):
    t = list(map(int, text.split())); n, m = t[:2]
    d = [[0 if i == j else 10 ** 9 for j in range(n)] for i in range(n)]
    for i in range(m):
        u, v, w = t[2 + 3 * i:5 + 3 * i]
        if w > 300:
            return "Impossible!"
        d[u][v] = d[v][u] = min(d[u][v], w)
    for c in range(n):
        for a in range(n):
            for b in range(n):
                d[a][b] = min(d[a][b], d[a][c] + d[c][b])
    best = 10 ** 9
    for who in itertools.product(range(4), repeat=n - 1):
        load = [0] * 4
        for v, g in enumerate(who):
            load[g] += d[0][v + 1]
        best = min(best, max(load))
    return str(2 * best) if best <= 300 else "Impossible!"


def gen_iati21a12(big):
    n = R.randint(1, 80 if big else 7)
    top = 1000 if big else 5
    rows = [(R.randint(1, i - 1), R.randint(-top if R.random() < .3 else 0, top)) for i in range(2, n + 1)]
    return lines((n,), [R.randint(0, top) for _ in range(n)], [R.randint(0, top) for _ in range(n)], *rows)


def gen_iati21a23(big):
    return lines((R.randint(1, 60 if big else 8), R.randint(0, 6), R.randint(0, 6)))


def valid_iati21a23(text, out):
    n, s, c = map(int, text.split()); t = list(map(int, out.split()))
    if not t or t[0] != len(t) - 1:
        return "bad length line"
    alive = [True] * n
    for i, shot in enumerate(t[1:]):
        x = shot - 1
        if not 0 <= x < n:
            return "shot off the board"
        away = i % (s + c) < s
        nxt = [False] * n
        for j in range(n):
            if alive[j] and j != x:
                step = (-1 if j < x else 1) * (1 if away else -1)
                nxt[min(n - 1, max(0, j + step))] = True
        alive = nxt
    return "a cell survives" if any(alive) else None


def gen_neerc10k(big):
    n = R.choice(range(1, 42 if big else 10, 2))
    es = set(tree(n))
    for _ in range(R.randint(0, 3 * n if big else n)):
        u, v = R.randint(1, n), R.randint(1, n)
        if u != v and (u, v) not in es and (v, u) not in es:
            es.add((u, v))
    if n >= 5 and R.random() < .3:            # complete graph: max degree even, every colour needed
        es = {(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)}
    es = list(es); R.shuffle(es)
    return lines((n, len(es)), *es)


def valid_neerc10k(text, out):
    t = list(map(int, text.split())); n, m = t[:2]; o = list(map(int, out.split()))
    deg = [0] * (n + 1)
    for i in range(m):
        deg[t[2 + 2 * i]] += 1; deg[t[3 + 2 * i]] += 1
    k = max(deg) | 1
    if len(o) != n + 1 or o[0] != k:
        return f"expected k = {k} and {n} colours"
    if not all(1 <= c <= k for c in o[1:]):
        return "colour out of range"
    for i in range(m):
        if o[t[2 + 2 * i]] == o[t[3 + 2 * i]]:
            return "edge with equal colours"
    return None


def gen_rpeng2(big):
    n = R.randint(2, 14 if big else 5)
    return lines((n,), *[(R.choice("tx"), R.randint(-9, 9)) for _ in range(n)])


def gen_noimps22p5(big):
    n = R.randint(1, 80 if big else 7)
    s, open_, left = [], 0, n
    while left or open_:
        if left and (not open_ or R.random() < .55):
            s.append("("); open_ += 1; left -= 1
        else:
            s.append(")"); open_ -= 1
    x, y = R.choice([(0, 1), (1, 0), (1, 1)])
    return lines((n, x, y), ("".join(s),), [R.randint(1, 30) for _ in range(n)])


def main():
    args = sys.argv[1:]
    cases = int(args[args.index("--cases") + 1]) if "--cases" in args else 150
    names = [a for a in args if a in ORIG] or sorted(ORIG)
    tmp = tempfile.mkdtemp(prefix="batch45_")
    total = 0
    for name in names:
        build(name, tmp)
        gen = globals()[f"gen_{name}"]
        brute = globals().get(f"brute_{name}")
        valid = globals().get(f"valid_{name}")
        ran = skipped = bruted = 0
        for case in range(cases):
            big = case >= cases // 2
            text = gen(big)
            a = sh(f"{tmp}/{name}_orig", input=text, timeout=120)
            if a.returncode:                  # the accepted code itself rejects this input
                skipped += 1
                continue
            b = sh(f"{tmp}/{name}_cp", input=text, timeout=120)
            bad = None
            if b.returncode:
                bad = f"port crashed\n{b.stderr[-1500:]}"
            elif valid:
                bad = valid(text, b.stdout)
                if bad is None and valid(text, a.stdout):
                    bad = "validator rejects the ORIGINAL: " + valid(text, a.stdout)
            elif a.stdout.split() != b.stdout.split():
                bad = f"expected {a.stdout[:300]!r}\n     got {b.stdout[:300]!r}"
            if bad is None and brute and not big:
                want = brute(text)
                bruted += 1
                if want.split() != b.stdout.split():
                    bad = f"exhaustive answer {want[:300]!r}\n      port gave {b.stdout[:300]!r}"
            if bad:
                sys.exit(f"{name}: MISMATCH on case {case}\n--- input\n{text[:1500]}--- {bad}")
            ran += 1
        total += ran
        how = "validated" if valid else "agree"
        extra = (f", {bruted} also exhaustive" if brute else "") + (f", {skipped} rejected by the original" if skipped else "")
        print(f"{name:11} {ran:4} cases {how}{extra}")
    shutil.rmtree(tmp)
    print(f"all {total} cases pass")


if __name__ == "__main__":
    main()
