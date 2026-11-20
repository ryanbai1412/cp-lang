#!/usr/bin/env python3
"""Differential check for scout batch 50.

Builds every today/<name>.cp with the prototype compiler (on a copy, since
`cp.py build` stamps its input), compiles the result under ASan/UBSan, and
runs it against the original corpus submission on random small and medium
inputs. Where the answer is not unique a validator replaces the token diff,
and where a tiny exhaustive solver exists it is run on the small inputs too.

    python3 proto/ports/scout_batch_50/check.py [name ...] [--cases N]
"""
import itertools, os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "noip02p4": 5096459,
    "noi03p4": 5032732,
    "noi04p5": 3744270,
    "noi20p5": 3803796,
    "noip08p4": 5185830,
    "noi23p1": 5723056,
    "noi17p1": 5690633,
    "noi22p1": 4800882,
    "noi22p4": 4802121,
    "noip22p2": 5445681,
}
R = random.Random(50)
GEN, BRUTE, VALID, CLOSE, VERDICT = {}, {}, {}, {}, {}


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


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def parents(n):
    """parent of node i (2..n) is a smaller node"""
    return [R.randint(1, i - 1) for i in range(2, n + 1)]


# ---- generators gen(big) -> input text; brutes brute(text) -> output text ----

def gen_noip02p4(big):
    n = R.randint(1, 30 if big else 6)
    hi = R.choice([3, 10, 500])
    pts = [(R.randint(0, hi), R.randint(0, hi)) for _ in range(n)]
    return lines((n, R.randint(1, 3)), *pts)


def brute_noip02p4(text):
    """every assignment of points to k boxes; boxes must be pairwise disjoint (closed)"""
    t = list(map(int, text.split())); n, k = t[0], t[1]
    pts = [(t[2 + 2 * i], t[3 + 2 * i]) for i in range(n)]
    if n > 7:
        return None
    best = None
    for asg in itertools.product(range(k), repeat=n):
        boxes = []
        for g in range(k):
            ps = [p for p, a in zip(pts, asg) if a == g]
            if ps:
                boxes.append((min(p[0] for p in ps), max(p[0] for p in ps), min(p[1] for p in ps), max(p[1] for p in ps)))
        if any(a[0] <= b[1] and b[0] <= a[1] and a[2] <= b[3] and b[2] <= a[3] for a, b in itertools.combinations(boxes, 2)):
            continue
        area = sum((b[1] - b[0]) * (b[3] - b[2]) for b in boxes)
        best = area if best is None else min(best, area)
    return f"{best}\n"


def gen_noi03p4(big):
    n = R.randint(3, 300 if big else 8)
    wmax = R.choice([1, 5, 10**9])
    par = parents(n) if R.random() < 0.7 else list(range(1, n))
    edges = [(p, i + 2, R.randint(1, wmax)) for i, p in enumerate(par)]
    R.shuffle(edges)
    return lines((n, n - 1), *edges)


def brute_noi03p4(text):
    """all triples: dist(a, b) + min(dist(c, a), dist(c, b)), Floyd-Warshall"""
    t = list(map(int, text.split())); n = t[0]
    if n > 12:
        return None
    d = [[0 if i == j else 10**18 for j in range(n)] for i in range(n)]
    for i in range(n - 1):
        u, v, w = t[2 + 3 * i:5 + 3 * i]
        d[u - 1][v - 1] = d[v - 1][u - 1] = w
    for k in range(n):
        for i in range(n):
            for j in range(n):
                d[i][j] = min(d[i][j], d[i][k] + d[k][j])
    return f"{max(d[a][b] + min(d[c][a], d[c][b]) for a in range(n) for b in range(n) for c in range(n))}\n"


def gen_noi04p5(big):
    n = R.randint(1, 25 if big else 5)
    m = R.randint(n, 40 if big else 9)       # every main piece holds at least one part
    price = lambda: f"{R.randint(0, 50)}.{R.randint(0, 9)}"
    return f"{price()} {price()} {n} {m}\n"


def random_binary_tree(n):
    """n nodes, root 1, children listed as (left, right) with 0 for none"""
    kids = [[0, 0] for _ in range(n + 1)]
    for v in range(2, n + 1):
        while True:
            u, side = R.randint(1, v - 1), R.randint(0, 1)
            if not kids[u][side]:
                kids[u][side] = v
                break
    return kids[1:]


def gen_noi20p5(big):
    out = [(R.randint(1, 5),)]
    for _ in range(out[0][0]):
        q = R.randint(1, 40 if big else 8)
        out.append((q,))
        for _ in range(q):
            n = R.randint(1, R.choice([2, 3, 4, 9] if big else [2, 3, 4]))
            out.append((n,))
            out += random_binary_tree(n)
    return lines(*out)


def gen_noip08p4(big):
    n = R.randint(1, 60 if big else 7)
    p = list(range(1, n + 1))
    if R.random() < 0.5:
        R.shuffle(p)
    else:                       # mostly sortable: a few local swaps of a sorted run
        for _ in range(R.randint(0, n)):
            i, j = R.randrange(n), R.randrange(n)
            if abs(i - j) <= 3:
                p[i], p[j] = p[j], p[i]
    return lines((n,), p)


def brute_noip08p4(text):
    """depth-first over a < b < c < d with dead states memoised: the first script found is the least"""
    t = list(map(int, text.split())); n, p = t[0], t[1:]
    if n > 9:
        return None
    dead = set()

    def go(i, need, s0, s1):
        if need > n:
            return []
        key = (i, need, s0, s1)
        if key in dead:
            return None
        moves = []
        if i < n and (not s0 or s0[-1] > p[i]):
            moves.append(("a", (i + 1, need, s0 + (p[i],), s1)))
        if s0 and s0[-1] == need:
            moves.append(("b", (i, need + 1, s0[:-1], s1)))
        if i < n and (not s1 or s1[-1] > p[i]):
            moves.append(("c", (i + 1, need, s0, s1 + (p[i],))))
        if s1 and s1[-1] == need:
            moves.append(("d", (i, need + 1, s0, s1[:-1])))
        for op, nxt in moves:
            rest = go(*nxt)
            if rest is not None:
                return [op] + rest
        dead.add(key)
        return None

    ans = go(0, 1, (), ())
    return "0\n" if ans is None else " ".join(ans) + "\n"


def valid_noip08p4(text, out):
    """the script must be legal and emit 1..n in order; the original's script is
    legal but not always the least (3 2 4 1 5), so only the port is held to the brute"""
    t = list(map(int, text.split())); n, p = t[0], t[1:]
    ops = out.split()
    if ops == ["0"]:
        return None
    st, i, need = ([], []), 0, 1
    for op in ops:
        if op in ("a", "c"):
            if i >= n:
                return "push with no input left"
            st[op == "c"].append(p[i]); i += 1
        elif op in ("b", "d"):
            s = st[op == "d"]
            if not s or s[-1] != need:
                return f"pop of {s[-1] if s else None} when {need} is due"
            s.pop(); need += 1
        else:
            return f"unknown op {op}"
    return None if need == n + 1 else "not everything was emitted"


def verdict_noip08p4(out):
    return out.split() == ["0"]


def gen_noi23p1(big):
    size = R.choice([4, 8, 30] if not big else [8, 40, 1000])
    n = R.randint(1, 60 if big else 6)
    rows = []
    for _ in range(n):
        kind = R.choice([1, 1, 2, 2, 3])
        a, b = sorted((R.randint(1, size), R.randint(1, size)))
        if kind == 1:
            y = R.randint(1, size); rows.append((1, a, y, b, y))
        elif kind == 2:
            x = R.randint(1, size); rows.append((2, x, a, x, b))
        else:
            x = R.randint(1, size - (b - a)); y = R.randint(1, size - (b - a))
            rows.append((3, x, y, x + b - a, y + b - a))
    return lines((0, size, size, n), *rows)


def brute_noi23p1(text):
    t = list(map(int, text.split())); n = t[3]
    if t[1] > 40:
        return None
    cells = set()
    for i in range(n):
        k, x1, y1, x2, y2 = t[4 + 5 * i:9 + 5 * i]
        if k == 1:
            cells |= {(x, y1) for x in range(min(x1, x2), max(x1, x2) + 1)}
        elif k == 2:
            cells |= {(x1, y) for y in range(min(y1, y2), max(y1, y2) + 1)}
        else:
            cells |= {(x1 + d, y1 + d) for d in range(x2 - x1 + 1)}
    return f"{len(cells)}\n"


def gen_noi17p1(big):
    """the number never goes negative; carries and borrows are forced by +-1 next to long runs"""
    q = R.randint(1, 300 if big else 12)
    span = min(30 * q, R.choice([8, 70, 400]))
    amax = R.choice([1, 3, 10**9])
    x, rows = 0, []
    for _ in range(q):
        if R.random() < 0.45 and rows:
            rows.append((2, R.randint(0, span + 40)))
            continue
        a, b = R.randint(-amax, amax) or 1, R.randint(0, span)
        if R.random() < 0.3:
            a, b = R.choice([1, -1]), R.randint(0, 3)
        if x + (a << b) < 0:
            a = abs(a)
        x += a << b
        rows.append((1, a, b))
    return lines((q, 0, 0, 0), *rows)


def brute_noi17p1(text):
    rows = [list(map(int, l.split())) for l in text.strip().split("\n")[1:]]
    x, out = 0, []
    for r in rows:
        if r[0] == 1:
            x += r[1] << r[2]
        else:
            out.append(str(x >> r[1] & 1))
    return "\n".join(out) + "\n"


def gen_noi22p1(big):
    """few distinct values so majorities are common; labels are retired by a join"""
    n, q = R.randint(1, 6), R.randint(1, 120 if big else 14)
    vmax = R.choice([1, 2, 3, n + q])
    rows, live, size, nxt = [(n, q)], {}, 0, 0
    for i in range(1, n + 1):
        m = R.randint(0, 4)
        live[i] = m
        rows.append([m] + [R.randint(1, vmax) for _ in range(m)])
    nxt = n + 1
    for _ in range(q):
        k = R.choice([1, 1, 2, 3, 3, 4])
        ids = list(live)
        x = R.choice(ids)
        if k == 2 and live[x] == 0:
            k = 1
        if k == 4 and len(ids) < 2:
            k = 3
        if k == 1:
            rows.append((1, x, R.randint(1, vmax))); live[x] += 1
        elif k == 2:
            rows.append((2, x)); live[x] -= 1
        elif k == 3:
            pick = R.sample(ids, R.randint(1, len(ids)))
            if sum(live[i] for i in pick) == 0:
                rows.append((1, x, R.randint(1, vmax))); live[x] += 1
            else:
                rows.append([3, len(pick)] + pick)
        else:
            a, b = R.sample(ids, 2)
            rows.append((4, a, b, nxt))
            live[nxt] = live.pop(a) + live.pop(b); nxt += 1
    return lines(*rows)


def brute_noi22p1(text):
    rows = [list(map(int, l.split())) for l in text.strip().split("\n")]
    n = rows[0][0]
    seq = {i + 1: rows[i + 1][1:] for i in range(n)}
    out = []
    for r in rows[n + 1:]:
        if r[0] == 1:
            seq[r[1]].append(r[2])
        elif r[0] == 2:
            seq[r[1]].pop()
        elif r[0] == 3:
            every = [v for i in r[2:] for v in seq[i]]
            best = max(set(every), key=every.count)
            out.append(str(best if 2 * every.count(best) > len(every) else -1))
        else:
            seq[r[3]] = seq.pop(r[1]) + seq.pop(r[2])
    return "\n".join(out) + "\n"


def gen_noi22p4(big):
    """H is random; G is H with up to k leaves grown on it, or (a third of the time) grown and then rewired"""
    tests, k = R.randint(1, 6), R.randint(0, 5)
    rows = [(0, tests, k)]
    for _ in range(tests):
        n2 = R.randint(1, 40 if big else 5)
        par = [-1] + (parents(n2) if R.random() < 0.6 else [R.randint(max(1, i - 2), i - 1) for i in range(2, n2 + 1)])
        g = list(par)
        for _ in range(R.randint(0, k)):
            g.append(R.randint(1, len(g)))
        if R.random() < 0.35 and len(g) > 2:
            i = R.randrange(2, len(g))
            g[i] = R.randint(1, i)
        rows += [(len(g),), relabel(g), (n2,), relabel(par)]
    return lines(*rows)


def relabel(par):
    """shuffle node names so a parent need not come before its child"""
    n = len(par)
    name = list(range(1, n + 1)); R.shuffle(name)
    out = [0] * n
    for i, p in enumerate(par):
        out[name[i] - 1] = -1 if p == -1 else name[p - 1]
    return out


def brute_noi22p4(text):
    """every tree reachable from G by single leaf deletions, as canonical nested tuples"""
    t = list(map(int, text.split())); k, pos, out = t[2], 3, []

    def canon(par):
        kids = {}
        for i, p in enumerate(par):
            kids.setdefault(p, []).append(i + 1)
        def go(u):
            return tuple(sorted(go(v) for v in kids.get(u, [])))
        return go(kids[-1][0])

    def shrink(tree):              # all trees one leaf smaller
        for i, c in enumerate(tree):
            rest = tree[:i] + tree[i + 1:]
            if c == ():
                yield rest
            for d in shrink(c):
                yield tuple(sorted(rest + (d,)))

    for _ in range(t[1]):
        n1 = t[pos]; g = canon(t[pos + 1:pos + 1 + n1]); pos += 1 + n1
        n2 = t[pos]; h = canon(t[pos + 1:pos + 1 + n2]); pos += 1 + n2
        if n1 > 12:
            return None
        level = {g}
        for _ in range(n1 - n2):
            level = {d for x in level for d in shrink(x)}
        out.append("Yes" if h in level and n1 - n2 <= k else "No")
    return "\n".join(out) + "\n"


def gen_noip22p2(big):
    """k = 2n - 2 or 2n - 1 as in the real data; every value is dealt an even number of times"""
    rows = [(R.randint(1, 4),)]
    for _ in range(rows[0][0]):
        n = R.randint(2, 12 if big else 4)
        k = 2 * n - R.choice([1, 2, 2])
        half = R.randint(1, 150 if big else 10)
        deck = [R.randint(1, k) for _ in range(half)] * 2
        R.shuffle(deck)
        rows += [(n, len(deck), k), deck]
    return lines(*rows)


def valid_noip22p2(text, out):
    t = list(map(int, text.split())); o = out.split(); pos, at = 1, 0
    for _ in range(t[0]):
        n, m, k = t[pos:pos + 3]; deck = t[pos + 3:pos + 3 + m]; pos += 3 + m
        ops = int(o[at]); at += 1
        if not m <= ops <= 2 * m:
            return f"{ops} operations for {m} cards"
        st, dealt = [[] for _ in range(n)], 0
        for _ in range(ops):
            if o[at] == "1":
                s = int(o[at + 1]) - 1; at += 2
                if dealt == m:
                    return "deck is empty"
                if st[s] and st[s][-1] == deck[dealt]:
                    st[s].pop()
                else:
                    st[s].append(deck[dealt])
                dealt += 1
            else:
                a, b = int(o[at + 1]) - 1, int(o[at + 2]) - 1; at += 3
                if a == b or not st[a] or not st[b] or st[a][0] != st[b][0]:
                    return f"bad match of stacks {a + 1} and {b + 1}"
                st[a].pop(0); st[b].pop(0)
        if dealt != m or any(st):
            return "cards are left"
    return None if at == len(o) else "trailing output"



for _n, _f in list(globals().items()):
    for _p, _d in (("gen_", GEN), ("brute_", BRUTE), ("valid_", VALID), ("close_", CLOSE), ("verdict_", VERDICT)):
        if _n.startswith(_p):
            _d[_n[len(_p):]] = _f


def run(exe, text):
    r = subprocess.run([exe], input=text, capture_output=True, text=True, timeout=60)
    if r.returncode:
        return None, r.stderr[-800:]
    return r.stdout, ""


def same(name, a, b):
    if name in CLOSE:
        return CLOSE[name](a, b)
    return a.split() == b.split()


def check(name, cases, tmp):
    build(name, tmp)
    ok = skipped = bruted = 0
    seen = set()
    for i in range(cases):
        text = GEN[name](i >= cases // 2)
        want, err = run(f"{tmp}/{name}_orig", text)
        if want is None:
            skipped += 1          # the original itself rejects this input
            continue
        got, err = run(f"{tmp}/{name}_cp", text)
        if got is None:
            sys.exit(f"{name}: port crashed\n{err}\n--- input\n{text[:2000]}")
        if name in VALID:
            for who, out in (("original", want), ("port", got)):
                why = VALID[name](text, out)
                if why:
                    sys.exit(f"{name}: {who} output invalid: {why}\n--- input\n{text[:2000]}\n--- output\n{out[:1000]}")
            if name in VERDICT and VERDICT[name](want) != VERDICT[name](got):
                sys.exit(f"{name}: verdicts differ\n--- input\n{text[:2000]}\n--- want\n{want[:600]}\n--- got\n{got[:600]}")
        elif not same(name, got, want):
            sys.exit(f"{name}: mismatch\n--- input\n{text[:2000]}\n--- want\n{want[:600]}\n--- got\n{got[:600]}")
        if name in BRUTE:
            slow = BRUTE[name](text)
            if slow is not None:
                bruted += 1
                if not same(name, got, slow):
                    sys.exit(f"{name}: port disagrees with the exhaustive solver\n--- input\n{text[:2000]}\n--- brute\n{slow[:600]}\n--- got\n{got[:600]}")
        ok += 1
        seen.add(got)
    extra = (f", {bruted} also against an exhaustive solver" if bruted else "") + (f", {skipped} rejected by the original" if skipped else "")
    print(f"{name:10} {ok} cases pass, {len(seen)} distinct outputs{extra}", flush=True)
    return ok


def main():
    args = sys.argv[1:]
    cases = 150
    if "--cases" in args:
        i = args.index("--cases"); cases = int(args[i + 1]); del args[i:i + 2]
    names = args or list(ORIG)
    with tempfile.TemporaryDirectory() as tmp:
        total = sum(check(n, cases, tmp) for n in names)
    print(f"total: {total} cases pass")


if __name__ == "__main__":
    main()
