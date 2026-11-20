#!/usr/bin/env python3
"""Differential check for scout batch 49: each today/<problem>.cp against the
original corpus C++ on random small and medium inputs.

    python3 proto/ports/scout_batch_49/check.py [problem ...] [--cases N]

The ports are built with the prototype compiler (on a copy, since `cp.py
build` stamps its input) and compiled with ASan + UBSan; the originals are
built plain. Outputs are compared token by token. The seven grader-style
originals get a small stdin/stdout main appended that prints exactly what
the port prints. Where a cheap independent check exists it runs as well:
a brute-force solver on the small inputs (ioi17p2, ioi21p1, ioi12p2,
ioi18p2, ioi99p6) or a validator of the port's own output (ioi00p2, ioi15p5,
ioi14p4 task 2).
"""
import itertools, os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "ioi00p2": "4682555", "ioi01p3": "4806167", "ioi99p6": "4899527", "ioi14p4": "4684567",
    "ioi15p5": "5723358", "ioi17p2": "5724153", "ioi21p1": "4698803", "ioi22p4": "4783516",
    "ioi12p2": "4689474", "ioi18p2": "4731161",
}

READ = "static std::vector<int> cp_ints(int k) { std::vector<int> v(k); for (auto& x : v) scanf(\"%d\", &x); return v; }\n"
WRAP = {
"ioi14p4": r"""
int main() {
    int task, n; scanf("%d %d", &task, &n);
    auto a = cp_ints(n);
    if (task == 1) printf("%d\n", valid(n, a.data()));
    else if (task == 2) {
        std::vector<int> out(1 << 20);
        int len = replacement(n, a.data(), out.data());
        printf("%d\n", len);
        for (int i = 0; i < len; i++) printf("%d ", out[i]);
        printf("\n");
    } else printf("%d\n", countReplacement(n, a.data()));
}
""",
"ioi15p5": r"""
int main() {
    int n, m; scanf("%d", &n);
    auto a = cp_ints(n);
    scanf("%d", &m);
    std::vector<int> x(m), y(m), p(m), q(m);
    for (int i = 0; i < m; i++) scanf("%d %d", &x[i], &y[i]);
    int r = findSwapPairs(n, a.data(), m, x.data(), y.data(), p.data(), q.data());
    printf("%d\n", r);
    for (int i = 0; i < r; i++) printf("%d %d\n", p[i], q[i]);
}
""",
"ioi17p2": r"""
int main() {
    int n, m; scanf("%d %d", &n, &m);
    auto r = cp_ints(n); auto b = cp_ints(m);
    std::cout << min_total_length(r, b) << "\n";
}
""",
"ioi21p1": r"""
int main() {
    int n, q; scanf("%d", &n);
    auto c = cp_ints(n);
    scanf("%d", &q);
    std::vector<int> l(q), r(q), v(q);
    for (int i = 0; i < q; i++) scanf("%d %d %d", &l[i], &r[i], &v[i]);
    for (int x : distribute_candies(c, l, r, v)) printf("%d ", x);
    printf("\n");
}
""",
"ioi22p4": r"""
int main() {
    int n, m, q; scanf("%d %d %d", &n, &m, &q);
    auto p = cp_ints(n + m); auto a = cp_ints(m);
    init(n, m, p, a);
    while (q--) { int l, r; scanf("%d %d", &l, &r); printf("%d\n", count_ways(l, r)); }
}
""",
"ioi12p2": r"""
int main() {
    int n, calls; scanf("%d %d", &n, &calls);
    Init(n);
    while (calls--) {
        int u, v; scanf("%d %d", &u, &v);
        if (u == -1) printf("%d\n", CountCritical()); else Link(u, v);
    }
}
""",
"ioi18p2": r"""
int main() {
    int h, w, q; scanf("%d %d %d", &h, &w, &q);
    std::vector<int> r(h * w), c(h * w);
    for (int i = 0; i < h * w; i++) scanf("%d %d", &r[i], &c[i]);
    give_initial_chart(h, w, r, c);
    while (q--) { int a, b; scanf("%d %d", &a, &b); printf("%d\n", swap_seats(a, b)); }
}
""",
}
PRELUDE = {}


def rows(*ls):
    return "\n".join(" ".join(map(str, l)) for l in ls) + "\n"


# ---- generators: gen(r, big) -> input text ---------------------------------

def gen_ioi00p2(r, big):
    n = r.randint(1, 60 if big else 9)
    m = r.randint(1, 8 if big else 4)
    return rows([n, m, r.randint(2, 6 if big else 4)], [r.randint(1, m) for _ in range(n)])


def gen_ioi01p3(r, big):
    if r.random() < 0.5:
        return rows(["N", r.choice([1, 2, 701149020, r.randint(1, 701149020)])])
    # a random valid grid: place A..Y, each on a random cell that keeps the staircase
    ln, word = [0] * 5, [None] * 25
    for letter in range(25):
        row = r.choice([i for i in range(5) if ln[i] < 5 and (i == 0 or ln[i - 1] > ln[i])])
        word[row * 5 + ln[row]] = chr(65 + letter)
        ln[row] += 1
    return rows(["W", "".join(word)])


def gen_ioi99p6(r, big):
    m, n = (r.randint(1, 130), r.randint(1, 12)) if big else (r.randint(1, 6), r.randint(1, 6))
    k = r.randint(0, 10)
    base = r.randint(0, 30)
    return rows([m, n, k], *[[base + r.randint(0, r.choice([3, 12, 30])) for _ in range(m)] for _ in range(n)])


def gen_ioi14p4(r, big):
    task = r.randint(1, 3)
    n = r.randint(1, 40 if big else 7)
    s = r.randrange(n)
    a = [(i + s) % n + 1 for i in range(n)]
    hi = 10 ** 9 if task == 3 and r.random() < 0.5 else n + r.randint(1, 3 * n)
    for i in r.sample(range(n), r.randint(0, n)):
        a[i] = r.randint(n + 1, hi)
    while len(set(a)) < n:                       # make the replacements distinct
        seen = set()
        for i in range(n):
            if a[i] in seen: a[i] = r.randint(n + 1, hi + n)
            seen.add(a[i])
    if task != 2 and r.random() < 0.4:           # break it: a duplicate or a misplaced original
        i, j = r.randrange(n), r.randrange(n)
        if r.random() < 0.5: a[i] = a[j]
        else: a[i], a[j] = a[j], a[i]
    return rows([task, n], a)


def gen_ioi15p5(r, big):
    n = r.randint(1, 50 if big else 7)
    a = list(range(n))
    if r.random() < 0.85: r.shuffle(a)
    m = 3 * n
    return rows([n], a, [m], *[[r.randrange(n), r.randrange(n)] for _ in range(m)])


def gen_ioi17p2(r, big):
    n, m = (r.randint(1, 40), r.randint(1, 40)) if big else (r.randint(1, 5), r.randint(1, 5))
    pts = r.sample(range(0, r.choice([n + m, 4 * (n + m), 10 ** 9]) + 1), n + m)
    r.shuffle(pts)
    return rows([n, m], sorted(pts[:n]), sorted(pts[n:]))


def gen_ioi21p1(r, big):
    n, q = (r.randint(1, 40), r.randint(1, 60)) if big else (r.randint(1, 5), r.randint(1, 8))
    hi = r.choice([3, 10, 10 ** 9])
    ops = []
    for _ in range(q):
        l = r.randrange(n); rr = r.randint(l, n - 1)
        ops.append([l, rr, r.choice([-1, 1]) * r.randint(1, hi)])
    return rows([n], [r.randint(1, hi) for _ in range(n)], [q], *ops)


def gen_ioi22p4(r, big):
    n = r.randint(1, 40 if big else 5)
    par = [-1] + [r.randint(0, i - 1) for i in range(1, n)]
    src = [r.randrange(n) for _ in range(r.randint(1, 40 if big else 5))]
    src += [g for g in range(n) if g not in par and g not in src]     # every gate needs an input
    r.shuffle(src)
    m, q = len(src), r.randint(1, 30 if big else 6)
    qs = []
    for _ in range(q):
        l = r.randrange(m); qs.append([n + l, n + r.randint(l, m - 1)])
    return rows([n, m, q], par + src, [r.randint(0, 1) for _ in range(m)], *qs)


def gen_ioi12p2(r, big):
    n = r.randint(1, 40 if big else 7)
    pairs = [(u, v) for u in range(n) for v in range(u + 1, n)]
    r.shuffle(pairs)
    style = r.randrange(3)                       # 0 random, 1 mostly a path/cycle, 2 sparse
    if style == 1:
        order = r.sample(range(n), n)
        chain = [(order[i], order[i + 1]) for i in range(n - 1)] + ([(order[-1], order[0])] if n > 2 else [])
        pairs = chain + [p for p in pairs if p not in chain and (p[1], p[0]) not in chain][:r.randint(0, 3)]
    else:
        pairs = pairs[:r.randint(0, n + 3 if style == 0 else max(1, n // 2))]
    calls = []
    for u, v in pairs:
        calls.append(r.choice([[u, v], [v, u]]))
        if r.random() < 0.6: calls.append([-1, -1])
    calls.append([-1, -1])
    return rows([n, len(calls)], *calls)


def gen_ioi18p2(r, big):
    h, w = (r.randint(1, 9), r.randint(1, 9)) if big else (r.randint(1, 4), r.randint(1, 4))
    seats = [[x, y] for x in range(h) for y in range(w)]
    if r.random() < 0.3:                         # nearly sorted: many beautiful prefixes
        for _ in range(r.randint(0, 2)):
            i, j = r.randrange(h * w), r.randrange(h * w); seats[i], seats[j] = seats[j], seats[i]
    else:
        r.shuffle(seats)
    q = r.randint(1, 30 if big else 6)
    return rows([h, w, q], *seats, *[[r.randrange(h * w), r.randrange(h * w)] for _ in range(q)])


# ---- independent solvers (small inputs) and output validators --------------

def brute_ioi17p2(t):
    n, m = t[0], t[1]
    pts = sorted([(x, 0) for x in t[2:2 + n]] + [(x, 1) for x in t[2 + n:]])
    k = n + m
    if k > 10: return None
    # every optimal wiring uses only edges between differently coloured points; try all edge subsets
    # through a DP over "which points are covered", adding one red-blue wire at a time
    INF = float("inf")
    best = [INF] * (1 << k); best[0] = 0
    wires = [(1 << i | 1 << j, abs(pts[i][0] - pts[j][0])) for i in range(k) for j in range(i) if pts[i][1] != pts[j][1]]
    for _ in range(k):                           # Bellman-Ford style relaxation to a fixpoint
        for mask in range(1 << k):
            if best[mask] < INF:
                for bits, c in wires:
                    if best[mask] + c < best[mask | bits]: best[mask | bits] = best[mask] + c
    return [best[-1]]


def brute_ioi21p1(t):
    n = t[0]; c = t[1:1 + n]; q = t[1 + n]; a = [0] * n
    for i in range(q):
        l, r, v = t[2 + n + 3 * i:5 + n + 3 * i]
        for j in range(l, r + 1): a[j] = max(0, min(c[j], a[j] + v))
    return a


def brute_ioi12p2(t):
    n, calls = t[0], t[1]
    if n > 12: return None
    edges, out = [], []
    def chains(skip):
        deg = [0] * n; up = list(range(n))
        def find(x):
            while up[x] != x: x = up[x]
            return x
        for u, v in edges:
            if skip in (u, v): continue
            deg[u] += 1; deg[v] += 1
            if deg[u] > 2 or deg[v] > 2 or find(u) == find(v): return False
            up[find(u)] = find(v)
        return True
    for i in range(calls):
        u, v = t[2 + 2 * i], t[3 + 2 * i]
        if u == -1: out.append(sum(chains(x) for x in range(n)))
        else: edges.append((u, v))
    return out


def brute_ioi18p2(t):
    h, w, q = t[0], t[1], t[2]
    pos = [(t[3 + 2 * i], t[4 + 2 * i]) for i in range(h * w)]
    out = []
    for i in range(q):
        a, b = t[3 + 2 * h * w + 2 * i], t[4 + 2 * h * w + 2 * i]
        pos[a], pos[b] = pos[b], pos[a]
        r0 = r1 = pos[0][0]; c0 = c1 = pos[0][1]; good = 0
        for k, (x, y) in enumerate(pos):
            r0, r1, c0, c1 = min(r0, x), max(r1, x), min(c0, y), max(c1, y)
            good += (r1 - r0 + 1) * (c1 - c0 + 1) == k + 1
        out.append(good)
    return out


def brute_ioi99p6(t):
    """Only the best area is unique, so compare areas rather than corners."""
    m, n, k = t[0], t[1], t[2]
    if n * m > 40: return None
    g = [t[3 + i * m:3 + (i + 1) * m] for i in range(n)]
    best = 0
    for x1 in range(m):
        for x2 in range(x1, min(m, x1 + 100)):
            for y1 in range(n):
                for y2 in range(y1, n):
                    cells = [g[y][x] for y in range(y1, y2 + 1) for x in range(x1, x2 + 1)]
                    if max(cells) - min(cells) <= k: best = max(best, len(cells))
    return best


def validate(name, t, out):
    """Returns an error string when the port's own output is not a valid answer."""
    if name == "ioi99p6":
        want = brute_ioi99p6(t)
        x1, y1, x2, y2 = out
        if want is not None and (x2 - x1 + 1) * (y2 - y1 + 1) != want: return f"area is not the maximum {want}"
        m, n, k = t[0], t[1], t[2]
        cells = [t[3 + (n - y) * m + x - 1] for y in range(y1, y2 + 1) for x in range(x1, x2 + 1)]
        if max(cells) - min(cells) > k or x2 - x1 >= 100: return "rectangle is not flat enough"
    if name == "ioi00p2":
        n, m, w = t[0], t[1], t[2]; park = t[3:]
        at = 1
        for _ in range(out[0]):
            d = out[at]; mv = out[at + 1:at + 1 + 2 * d]; at += 1 + 2 * d
            if not 1 <= d <= w: return "bad driver count"
            src, dst = mv[0::2], mv[1::2]
            if len(set(src)) < d or len(set(dst)) < d or set(src) != set(dst): return "a round is not a set of cycles"
            new = park[:]
            for s, e in zip(src, dst): new[e - 1] = park[s - 1]
            park = new
        if at != len(out) or park != sorted(park): return "cars are not sorted at the end"
    if name == "ioi15p5":
        n = t[0]; a = t[1:1 + n]; m = t[1 + n]; e = t[2 + n:]
        if not 0 <= out[0] <= m or len(out) != 1 + 2 * out[0]: return "bad round count"
        for i in range(out[0]):
            for x, y in ((e[2 * i], e[2 * i + 1]), (out[1 + 2 * i], out[2 + 2 * i])):
                a[x], a[y] = a[y], a[x]
        if a != sorted(a): return "row is not sorted after the last round"
        # fewest rounds: no shorter prefix of Ermek's swaps leaves a row we could fix in time
        a = t[1:1 + n]
        for i in range(out[0]):
            seen, cyc = [False] * n, 0
            for s in range(n):
                if not seen[s]:
                    cyc += 1
                    while not seen[s]: seen[s] = True; s = a[s]
            if n - cyc <= i: return f"{i} rounds would have been enough"
            a[e[2 * i]], a[e[2 * i + 1]] = a[e[2 * i + 1]], a[e[2 * i]]
    if name == "ioi14p4" and t[0] == 2:
        n = t[1]; final = t[2:]
        s = next(((final[i] - 1 - i) % n for i in range(n) if final[i] <= n), n - 1)
        now = [(i + s) % n + 1 for i in range(n)]
        if out[0] != len(out) - 1: return "bad length"
        for step, g in enumerate(out[1:]):
            if g not in now: return "replaced a gondola that is not there"
            now[now.index(g)] = n + 1 + step
        if now != final: return "replacement sequence does not produce the input"
    return None


def sh(cmd):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True)


def build(name, tmp):
    shutil.copy(f"{HERE}/today/{name}.cp", f"{tmp}/{name}.cp")
    r = sh(f"python3 {ROOT}/proto/cp.py build {tmp}/{name}.cp")
    if not os.path.exists(f"{tmp}/{name}.cpp"): sys.exit(f"{name}: prototype build failed\n{r.stdout}{r.stderr}")
    flags = "-std=c++17 -O1 -g -w -fsanitize=address,undefined -fno-sanitize-recover=all"
    r = sh(f"g++ {flags} -o {tmp}/{name}_cp {tmp}/{name}.cpp")
    if r.returncode: sys.exit(f"{name}: emitted C++ failed to compile\n{r.stderr[:3000]}")
    src = open(f"{ROOT}/corpus/submissions/{ORIG[name]}.cpp", encoding="utf-8", errors="replace").read()
    if name in WRAP:
        # prototypes the grader header would have supplied go right after the template's last macro
        head, sep, tail = src.partition("#define sz(")
        line, _, rest = tail.partition("\n")
        src = head + sep + line + "\n" + PRELUDE.get(name, "") + rest + READ + WRAP[name]
    open(f"{tmp}/{name}_orig.cpp", "w").write(src)
    r = sh(f"g++ -std=c++17 -O1 -w -o {tmp}/{name}_orig {tmp}/{name}_orig.cpp")
    if r.returncode: sys.exit(f"{name}: original failed to compile\n{r.stderr[:3000]}")


def main():
    args = sys.argv[1:]
    cases = 300
    if "--cases" in args:
        i = args.index("--cases"); cases = int(args[i + 1]); del args[i:i + 2]
    env = dict(os.environ, ASAN_OPTIONS="detect_leaks=0:detect_stack_use_after_return=0")
    with tempfile.TemporaryDirectory() as tmp:
        for name in args or list(ORIG):
            build(name, tmp)
            rng = random.Random(49_000 + sum(map(ord, name)))
            gen, brute = globals()["gen_" + name], globals().get("brute_" + name)
            brute = None if name == "ioi99p6" else brute
            solved = 0
            for case in range(cases):
                inp = gen(rng, case >= cases * 2 // 3)          # last third: medium inputs
                def fail(msg):
                    open(f"/tmp/{name}_fail.in", "w").write(inp)
                    sys.exit(f"{name} case {case}: {msg} (input in /tmp/{name}_fail.in)")
                outs = []
                for side in ("cp", "orig"):
                    r = subprocess.run(f"ulimit -s 1048576; {tmp}/{name}_{side}", shell=True, input=inp,
                                       capture_output=True, text=True, env=env, timeout=120)
                    if r.returncode: fail(f"{side} exited {r.returncode}\n{r.stderr[-2000:]}")
                    outs.append(r.stdout.split())
                if outs[0] != outs[1]:
                    fail(f"outputs differ\nport: {' '.join(outs[0])[:400]}\norig: {' '.join(outs[1])[:400]}")
                if name == "ioi01p3": continue
                toks = [int(x) for x in inp.split()]
                got = [int(x) for x in outs[0]]
                err = validate(name, toks, got)
                if err: fail(f"port output is invalid: {err}")
                want = brute(toks) if brute else None
                if want is not None:
                    solved += 1
                    if want != got: fail(f"port disagrees with the brute force\nport: {got[:60]}\nbrute: {want[:60]}")
            extra = f", {solved} also against a brute force" if solved else ""
            print(f"{name}: {cases} cases agree ({cases - cases * 2 // 3} medium{extra})", flush=True)


if __name__ == "__main__":
    main()
