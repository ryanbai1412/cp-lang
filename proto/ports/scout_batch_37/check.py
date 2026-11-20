#!/usr/bin/env python3
"""Differential check for scout batch 37: each today/<problem>.cp against the
original corpus C++ on random small and medium inputs.

    python3 proto/ports/scout_batch_37/check.py [problem ...] [--cases N]

The ports are built with ASan + UBSan, the originals with plain -O1. Outputs are compared token by token.
Grader-style originals (ioi15p1, ioi20p2, ioi12p5) get a small stdin/stdout
main appended; the wrapper prints exactly what the port prints.
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "noi10p1": "5670455", "ioi15p1": "5723345", "noi14p5": "3601452", "noip18p4": "4004140",
    "ioi09p5": "5106592", "noip06p3": "5165783", "noip10p4": "5205786", "ioi20p2": "4724578",
    "ioi12p5": "4951033", "noip04p4": "5117252",
}

WRAP = {
"ioi15p1": r"""
int main() {
    int n, k, l; scanf("%d %d %d", &n, &k, &l);
    std::vector<int> p(n); for (auto& x : p) scanf("%d", &x);
    std::cout << delivery(n, k, l, p.data()) << "\n";
}
""",
"ioi20p2": r"""
static std::vector<std::vector<int>> cp_built;
void build(std::vector<std::vector<int>> b) { cp_built = b; }
int main() {
    int n; scanf("%d", &n);
    std::vector<std::vector<int>> p(n, std::vector<int>(n));
    for (auto& r : p) for (auto& x : r) scanf("%d", &x);
    int ok = construct(p);
    printf("%d\n", ok);
    if (ok) for (auto& r : cp_built) { for (int x : r) printf("%d ", x); printf("\n"); }
}
""",
"ioi12p5": r"""
static std::vector<int> cp_bits, cp_req, cp_back; static int cp_at = 0;
void WriteAdvice(unsigned char a) { cp_bits.push_back(a); }
int GetRequest() { return cp_req[cp_at++]; }
void PutBack(int t) { cp_back.push_back(t); }
int main() {
    int n, k; scanf("%d %d", &n, &k);
    cp_req.resize(n); for (auto& x : cp_req) scanf("%d", &x);
    std::vector<int> c = cp_req;
    ComputeAdvice(c.data(), n, k, 2);
    std::vector<unsigned char> buf(cp_bits.begin(), cp_bits.end());
    Assist(buf.data(), n, k, (int)buf.size());
    for (int b : cp_bits) printf("%d ", b); printf("\n");
    for (int b : cp_back) printf("%d ", b); printf("\n");
}
""",
}
PRELUDE = {"ioi15p1": "typedef long long ll;\n"}

def rows(*ls): return "\n".join(" ".join(map(str, l)) for l in ls) + "\n"

def gen_noi10p1(r, big):
    hi = 100000 if big else 40
    return rows([r.randint(1, hi), r.randint(1, hi)])

def gen_ioi15p1(r, big):
    n = r.randint(1, 2000 if big else 9); k = r.randint(1, n)
    l = r.randint(1, 10**9 if big else 12)
    return rows([n, k, l], sorted(r.randint(0, l - 1) for _ in range(n)))

def gen_noi14p5(r, big):
    n, m = (r.randint(1, 60), r.randint(1, 60)) if big else (r.randint(1, 5), r.randint(1, 5))
    q = r.randint(0, 50 if big else 4)
    d = r.randint(1, 10**8)
    head = [r.randint(0, d - 1), r.randint(0, 300), r.randint(0, 10**8), r.randint(0, 10**8), d]
    return rows(head, [n, m, q], *[[r.randint(1, n * m), r.randint(1, n * m)] for _ in range(q)])

def gen_noip18p4(r, big):
    n = r.randint(1, 300 if big else 8)
    edges = [(r.randint(1, v - 1), v) for v in range(2, n + 1)]
    if n >= 3 and r.random() < 0.6:
        while True:
            u, v = sorted(r.sample(range(1, n + 1), 2))
            if (u, v) not in edges: break
        edges.append((u, v))
    relabel = list(range(1, n + 1)); r.shuffle(relabel)
    edges = [(relabel[u - 1], relabel[v - 1]) if r.random() < 0.5 else (relabel[v - 1], relabel[u - 1]) for u, v in edges]
    r.shuffle(edges)
    return rows([n, len(edges)], *edges)

def gen_ioi09p5(r, big):
    n = r.randint(1, 100 if big else 3); m = r.randint(1, 2000 if big else 6)
    ev, parked, queue, todo = [], [], [], list(range(1, m + 1)); r.shuffle(todo)
    while todo or parked:                      # only parked cars may leave
        if todo and (not parked or r.random() < 0.55):
            queue.append(todo.pop()); ev.append(queue[-1])
        else:
            ev.append(-parked.pop(r.randrange(len(parked))))
        while queue and len(parked) < n: parked.append(queue.pop(0))
    return rows([n, m], *[[r.randint(1, 100)] for _ in range(n)], *[[r.randint(1, 10000)] for _ in range(m)], *[[e] for e in ev])

def gen_noip06p3(r, big):
    m, n = (r.randint(1, 19), r.randint(1, 19)) if big else (r.randint(1, 4), r.randint(1, 4))
    order = [j for j in range(1, n + 1) for _ in range(m)]; r.shuffle(order)
    mach = [r.sample(range(1, m + 1), m) for _ in range(n)]
    return rows([m, n], order, *mach, *[[r.randint(1, 20) for _ in range(m)] for _ in range(n)])

def gen_noip10p4(r, big):
    n, m = (r.randint(1, 60), r.randint(1, 60)) if big else (r.randint(1, 5), r.randint(1, 6))
    style = r.randint(0, 2)
    if style == 0: g = [[r.randint(1, 10) for _ in range(m)] for _ in range(n)]
    elif style == 1:   # sloping terrain: most cities get water
        g = [[3 * (n - i) + r.randint(0, 3) for _ in range(m)] for i in range(n)]
    else:
        g = [[10 * (n - i) + abs(j - m // 2) + r.randint(0, 2) for j in range(m)] for i in range(n)]
    return rows([n, m], *g)

def gen_ioi20p2(r, big):
    n = r.randint(1, 60 if big else 7)
    # plant a tree / cycle layout, then sometimes corrupt it
    tree = [r.randint(0, max(0, n // 3)) for _ in range(n)]
    ring = {t: r.randint(0, max(0, n // 6)) for t in set(tree)}
    p = [[1 if tree[i] == tree[j] else 2 if ring[tree[i]] == ring[tree[j]] else 0 for j in range(n)] for i in range(n)]
    for _ in range(r.choice([0, 0, 0, 1, 2])):
        i, j = r.randrange(n), r.randrange(n)
        if i != j: p[i][j] = p[j][i] = r.randint(0, 3)
    perm = list(range(n)); r.shuffle(perm)
    return rows([n], *[[p[perm[i]][perm[j]] for j in range(n)] for i in range(n)])

def gen_ioi12p5(r, big):
    n = r.randint(1, 400 if big else 9); k = r.randint(1, n)
    span = r.randint(1, n)
    return rows([n, k], [r.randrange(span) for _ in range(n)])

def gen_noip04p4(r, big):
    while True:
        n = r.randint(2, 13 if big else 6)
        digit = list(range(n)); r.shuffle(digit)          # digit[letter]
        letter = {d: chr(65 + l) for l, d in enumerate(digit)}
        a = [r.randrange(n) for _ in range(n)]; b = [r.randrange(n) for _ in range(n)]
        c, carry = [], 0
        for i in range(n - 1, -1, -1):
            s = a[i] + b[i] + carry; c.append(s % n); carry = s // n
        if carry: continue
        c.reverse()
        if len(set(a + b + c)) < n: continue             # every letter appears
        return "%d\n%s\n%s\n%s\n" % (n, *["".join(letter[d] for d in w) for w in (a, b, c)])

def sh(cmd, **kw): return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)

def build(name, tmp):
    cp = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), cp)   # cp.py build rewrites its input
    r = sh(f"python3 {ROOT}/proto/cp.py build {cp}")
    if r.returncode: sys.exit(f"{name}: cp build failed\n{r.stdout}{r.stderr}")
    flags = "-std=c++17 -O1 -g -w -fsanitize=address,undefined -fno-sanitize-recover=all"
    r = sh(f"g++ {flags} -o {tmp}/{name}_cp {tmp}/{name}.cpp")
    if r.returncode: sys.exit(f"{name}: port g++ failed\n{r.stderr[:3000]}")
    src = PRELUDE.get(name, "") + open(f"{ROOT}/corpus/submissions/{ORIG[name]}.cpp").read() + WRAP.get(name, "")
    open(f"{tmp}/{name}_orig.cpp", "w").write(src)
    # originals are built plain: 4004140 (noip18p4) trips ASan on a dangling minmax reference
    r = sh(f"g++ -std=c++17 -O1 -w -o {tmp}/{name}_orig {tmp}/{name}_orig.cpp")
    if r.returncode: sys.exit(f"{name}: original g++ failed\n{r.stderr[:3000]}")

def main():
    args = sys.argv[1:]
    cases = 300
    if "--cases" in args:
        i = args.index("--cases"); cases = int(args[i + 1]); del args[i:i + 2]
    names = args or list(ORIG)
    env = dict(os.environ, ASAN_OPTIONS="detect_leaks=0:detect_stack_use_after_return=0")
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            build(name, tmp)
            rng = random.Random(37_000 + sum(map(ord, name)))
            gen = globals()["gen_" + name]
            for case in range(cases):
                big = case >= cases * 2 // 3          # last third: medium inputs
                inp = gen(rng, big)
                outs = []
                for side in ("cp", "orig"):
                    r = subprocess.run(f"ulimit -s 1048576; {tmp}/{name}_{side}", shell=True, input=inp,
                                       capture_output=True, text=True, env=env, timeout=120)
                    if r.returncode:
                        open(f"/tmp/{name}_fail.in", "w").write(inp)
                        sys.exit(f"{name} case {case}: {side} exited {r.returncode} (input in /tmp/{name}_fail.in)\n{r.stderr[-2000:]}")
                    outs.append(r.stdout.split())
                if outs[0] != outs[1]:
                    open(f"/tmp/{name}_fail.in", "w").write(inp)
                    sys.exit(f"{name} case {case}: outputs differ (input in /tmp/{name}_fail.in)\n"
                             f"port: {' '.join(outs[0])[:400]}\norig: {' '.join(outs[1])[:400]}")
            print(f"{name}: {cases} cases agree ({cases - cases * 2 // 3} medium)", flush=True)

if __name__ == "__main__":
    main()
