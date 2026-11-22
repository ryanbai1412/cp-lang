#!/usr/bin/env python3
"""Differential test of the blind-critic-4 ports against the original C++ submissions.

Usage (from anywhere): python3 proto/ports/blind_critic_4/check.py [name ...] [--iters N]
Builds scratch copies under a temp dir, so the tracked ports are never stamped.
"""
import os, random, resource, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(os.path.dirname(HERE))
CORPUS = os.path.join(os.path.dirname(PROTO), "corpus", "submissions")

ORIGINAL = {
    "ioi11p5": 3469626,
    "ioi22p1": 4785724,
    "ioi20p1": 5725518,
    "ioi17p6": 5659304,
    "ioi13p6": 4688844,
    "ioi15p6": 5723344,
    "ioi11p6": 3735613,
    "dmopc21c5p5": 4223999,
}

# Thin C++ grader drivers so the original IOI grader submissions run over the same stdin/stdout format.
DRIVERS = {
    "ioi11p5": r"""
#define main _orig_main
#include "{orig}"
#undef main
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int n, l, q;
    if (!(cin >> n >> l >> q)) return 0;
    vector<int> a(n);
    for (int i = 0; i < n; i++) cin >> a[i];
    init(n, l, a.data());
    for (int i = 0; i < q; i++) {
        int idx, y; cin >> idx >> y;
        cout << update(idx, y) << "\n";
    }
    return 0;
}
""",
    "ioi22p1": r"""
#include "{orig}"
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int n, m;
    if (!(cin >> n >> m)) return 0;
    vector<int> xs(m), ys(m), ws(m);
    for (int i = 0; i < m; i++) cin >> xs[i] >> ys[i] >> ws[i];
    cout << max_weights(n, m, xs, ys, ws) << "\n";
    return 0;
}
""",
    "ioi20p1": r"""
#include "{orig}"
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int n, k, q;
    if (!(cin >> n >> k >> q)) return 0;
    vector<int> r(n);
    for (int i = 0; i < n; i++) cin >> r[i];
    init(k, r);
    for (int i = 0; i < q; i++) {
        int x, y; cin >> x >> y;
        cout << compare_plants(x, y) << "\n";
    }
    return 0;
}
""",
    "ioi17p6": r"""
#include "{orig}"
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int n, s;
    if (!(cin >> n >> s)) return 0;
    vector<int> p(n);
    for (int i = 0; i < n; i++) cin >> p[i];
    cout << minimum_walk(p, s) << "\n";
    return 0;
}
""",
    "ioi13p6": r"""
#include "{orig}"
#undef long
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int R, C, N;
    if (!(cin >> R >> C >> N)) return 0;
    init(R, C);
    for (int i = 0; i < N; i++) {
        int op; cin >> op;
        if (op == 1) {
            int p, q; int64_t k; cin >> p >> q >> k;
            update(p, q, k);
        } else {
            int p, q, u, v; cin >> p >> q >> u >> v;
            cout << calculate(p, q, u, v) << "\n";
        }
    }
    return 0;
}
""",
    "ioi15p6": r"""
#include <bits/stdc++.h>
using namespace std;
static int _N_leaves, _M_nodes;
static vector<vector<pair<int,int>>> _adj;
static vector<vector<int>> _tdist;
int getDistance(int i, int j) {
    return _tdist[i][j];
}
#include "{orig}"
#undef long
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    if (!(cin >> _N_leaves >> _M_nodes)) return 0;
    _adj.assign(_M_nodes, {});
    for (int i = 0; i < _M_nodes - 1; i++) {
        int u, v, w; cin >> u >> v >> w;
        _adj[u].push_back({v, w});
        _adj[v].push_back({u, w});
    }
    _tdist.assign(_N_leaves, vector<int>(_N_leaves, 0));
    for (int s = 0; s < _N_leaves; s++) {
        vector<int> d(_M_nodes, -1);
        queue<int> q;
        d[s] = 0; q.push(s);
        while (!q.empty()) {
            int u = q.front(); q.pop();
            for (auto [v, w] : _adj[u]) {
                if (d[v] == -1) { d[v] = d[u] + w; q.push(v); }
            }
        }
        for (int t = 0; t < _N_leaves; t++) _tdist[s][t] = d[t];
    }
    cout << hubDistance(_N_leaves, 5) << "\n";
    return 0;
}
""",
    "ioi11p6": r"""
#include <bits/stdc++.h>
using namespace std;
static vector<int> _sent, _out;
void send(int b) { _sent.push_back(b); }
void output(int b) { _out.push_back(b); }
#include "{orig}"
#undef long
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<int> msg(n);
    for (int i = 0; i < n; i++) cin >> msg[i];
    _sent.clear(); _out.clear();
    encode(n, msg.data());
    reverse(_sent.begin(), _sent.end());
    decode(n, (int)_sent.size(), _sent.data());
    cout << _sent.size() << "\n";
    for (int b : _out) cout << b << "\n";
    return 0;
}
""",
}


def gen_ioi11p5(it):
    n = random.randint(2, 15 if it % 5 else 80)
    l = random.randint(0, 20 if it % 3 else 10**6)
    q = random.randint(1, 25 if it % 5 else 80)
    xs = sorted(random.randint(0, 40 if it % 3 else 10**6) for _ in range(n))
    out = [f"{n} {l} {q}", " ".join(map(str, xs))]
    for _ in range(q):
        out.append(f"{random.randint(0, n - 1)} {random.randint(0, 40 if it % 3 else 10**6)}")
    return "\n".join(out) + "\n"


def gen_ioi22p1(it):
    n = random.randint(2, 10 if it % 5 else 40)
    max_m = min(n * n, 18 if it % 5 else 120)
    m = random.randint(1, max_m)
    coords = random.sample([(x, y) for x in range(n) for y in range(n)], m)
    out = [f"{n} {m}"]
    for x, y in coords:
        out.append(f"{x} {y} {random.randint(1, 50 if it % 2 else 10**9)}")
    return "\n".join(out) + "\n"


def gen_ioi20p1(it):
    n = random.randint(2, 14 if it % 5 else 60)
    k = random.randint(2, n)
    q = random.randint(1, 25 if it % 5 else 80)
    # Generate a valid height permutation first, then derive r[i] from it!
    h = list(range(n))
    random.shuffle(h)
    r = [sum(1 for d in range(1, k) if h[(i + d) % n] > h[i]) for i in range(n)]
    out = [f"{n} {k} {q}", " ".join(map(str, r))]
    for _ in range(q):
        x, y = random.sample(range(n), 2)
        out.append(f"{x} {y}")
    return "\n".join(out) + "\n"


def gen_ioi17p6(it):
    n = random.randint(1, 12 if it % 5 else 100)
    s = random.randint(0, n - 1)
    p = list(range(n))
    if it % 7 != 0:
        random.shuffle(p)
        # Occasionally force fixed points on prefixes/suffixes (crucial edge case!)
        if it % 3 == 0 and n >= 4:
            p[0] = 0
            idx = p.index(0)
            p[idx] = 0
            # Re-make a clean permutation on inner subarray
            l = random.randint(0, n // 3)
            r = random.randint(2 * n // 3, n - 1)
            sub = list(range(l, r + 1))
            random.shuffle(sub)
            p = list(range(n))
            p[l : r + 1] = sub
    return f"{n} {s}\n" + " ".join(map(str, p)) + "\n"


def gen_ioi13p6(it):
    R = random.choice([1, 5, 12, 10**9])
    C = random.choice([1, 5, 12, 10**9])
    N = random.randint(1, 30 if it % 5 else 100)
    out = [f"{R} {C} {N}"]
    coords_used = []
    for _ in range(N):
        if not coords_used or random.random() < 0.55:
            if coords_used and random.random() < 0.25:
                r, c = random.choice(coords_used)
            else:
                r = random.randint(0, R - 1)
                c = random.randint(0, C - 1)
                coords_used.append((r, c))
            val = random.choice([0, 1, 6, 12, 18, 24, random.randint(1, 10**12)])
            out.append(f"1 {r} {c} {val}")
        else:
            r1 = random.randint(0, R - 1)
            r2 = random.randint(r1, R - 1)
            c1 = random.randint(0, C - 1)
            c2 = random.randint(c1, C - 1)
            if coords_used and random.random() < 0.5:
                cr, cc = random.choice(coords_used)
                r1 = min(r1, cr); r2 = max(r2, cr)
                c1 = min(c1, cc); c2 = max(c2, cc)
            out.append(f"2 {r1} {c1} {r2} {c2}")
    return "\n".join(out) + "\n"


def gen_ioi15p6(it):
    # Weighted tree where nodes 0..n-1 are leaves (degree 1) and internal nodes have degree >= 3.
    n = random.randint(3, 12 if it % 5 else 45)
    # Build by starting with 1 internal node attached to 3 leaves, then either attaching a new leaf
    # to an existing internal node OR splitting an edge with a new internal node + new leaf.
    # For clean numbering, put leaves at 0..n-1 and internal nodes at n..m-1.
    edges = [(0, n), (1, n), (2, n)]
    next_int = n + 1
    for leaf in range(3, n):
        if random.random() < 0.45:
            # Attach to an existing internal node (creates high-degree hubs to stress Boyer-Moore!)
            hub = random.randint(n, next_int - 1)
            edges.append((leaf, hub))
        else:
            # Split an existing edge with a new internal node
            ei = random.randrange(len(edges))
            u, v = edges[ei]
            ni = next_int
            next_int += 1
            edges[ei] = (u, ni)
            edges.append((ni, v))
            edges.append((leaf, ni))
    m = next_int
    out = [f"{n} {m}"]
    for u, v in edges:
        out.append(f"{u} {v} {random.randint(1, 20 if it % 2 else 1000)}")
    return "\n".join(out) + "\n"


def gen_ioi11p6(it):
    n = random.randint(1, 16 if it % 3 else 64)
    if it % 7 == 0:
        msg = [255] * n
    elif it % 7 == 1:
        msg = [0] * n
    else:
        msg = [random.randint(0, 255) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, msg)) + "\n"


def gen_dmopc21c5p5(it):
    if it < 12:
        n = it + 1
    else:
        n = random.randint(13, 60)
    return f"{n}\n"


def score_dmopc21c5p5(n, out_tokens):
    vals = list(map(int, out_tokens))
    if len(vals) != n or sorted(vals) != list(range(1, n + 1)):
        return None
    def is_prime(x):
        if x < 2: return False
        d = 2
        while d * d <= x:
            if x % d == 0: return False
            d += 1
        return True
    return sum(abs(vals[i] - vals[i + 1]) for i in range(n - 1) if is_prime(abs(vals[i] - vals[i + 1])))


def compile_ref(name, orig_path, tmp):
    ref_bin = os.path.join(tmp, f"{name}_ref")
    if name in DRIVERS:
        wrap = os.path.join(tmp, f"{name}_driver.cpp")
        with open(wrap, "w") as f:
            f.write(DRIVERS[name].replace("{orig}", orig_path))
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ref_bin, wrap], check=True)
    else:
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ref_bin, orig_path], check=True)
    return ref_bin


def main():
    args = sys.argv[1:]
    iters = 200
    if "--iters" in args:
        i = args.index("--iters")
        iters = int(args[i + 1])
        args = args[:i] + args[i + 2:]
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    tmp = tempfile.mkdtemp(prefix="bc4_")
    bad = 0
    for name in args or ORIGINAL:
        orig = ORIGINAL[name]
        cp = os.path.join(tmp, name + ".cp")
        shutil.copy(os.path.join(HERE, name + ".cp"), cp)
        r = subprocess.run([sys.executable, os.path.join(PROTO, "cp.py"), "build", cp], capture_output=True, text=True)
        if r.returncode:
            print(f"BUILD FAIL {name}:\n{r.stderr[-1500:]}")
            bad += 1
            continue
        ours = os.path.join(tmp, name + "_bin")
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ours, os.path.join(tmp, name + ".cpp")], check=True)
        ref = compile_ref(name, os.path.join(CORPUS, f"{orig}.cpp"), tmp)
        gen = globals()["gen_" + name]
        n_iters = min(iters, 30) if name == "dmopc21c5p5" else iters
        dmopc_exact_ok = 0
        dmopc_heur_sum = 0
        dmopc_ref_sum = 0
        for it in range(n_iters):
            random.seed(it)
            inp = gen(it)
            a = subprocess.run([ours], input=inp, capture_output=True, text=True, timeout=20)
            b = subprocess.run([ref], input=inp, capture_output=True, text=True, timeout=20)
            if name == "dmopc21c5p5":
                n = int(inp.strip())
                sa = score_dmopc21c5p5(n, a.stdout.split())
                sb = score_dmopc21c5p5(n, b.stdout.split())
                if a.returncode or sa is None or (n <= 12 and sa < sb):
                    print(f"MISMATCH {name} n={n} ours={sa} ref={sb}")
                    bad += 1
                    break
                if n <= 12:
                    dmopc_exact_ok += 1
                else:
                    dmopc_heur_sum += sa
                    dmopc_ref_sum += sb
            elif name == "ioi11p6":
                # Check both round-trip byte equality AND channel length <= 5*n!
                toks_a = a.stdout.split()
                toks_b = b.stdout.split()
                n = int(inp.split()[0])
                if a.returncode or int(toks_a[0]) > 5 * n or toks_a[1:] != toks_b[1:]:
                    print(f"MISMATCH {name} it={it}\nours={toks_a[:20]}\nref={toks_b[:20]}")
                    bad += 1
                    break
            else:
                if a.returncode or a.stdout.split() != b.stdout.split():
                    print(f"MISMATCH {name} it={it}\ninput:\n{inp[:600]}\nours:\n{a.stdout[:300]}{a.stderr[:300]}\nref:\n{b.stdout[:300]}")
                    bad += 1
                    break
        else:
            if name == "dmopc21c5p5":
                ratio = (dmopc_heur_sum / dmopc_ref_sum) if dmopc_ref_sum else 1.0
                print(f"OK {name}: {dmopc_exact_ok} exact Subset-DP cases (n=1..12, >= ref) + {n_iters - dmopc_exact_ok} heuristic cases (valid perms, score ratio {ratio:.4f} vs {orig}.cpp)")
            else:
                print(f"OK {name}: {n_iters} tests agree with {orig}.cpp")
    shutil.rmtree(tmp)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
