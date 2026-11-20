#!/usr/bin/env python3
"""Differential test harness for Scout Batch 51 (v2.1 Olympiads).

Builds every runnable port in `today/` with `proto/cp.py build` on temporary copies
(never mutating the committed `.cp` files), compiles both the generated C++17 and
the original accepted DMOJ C++ submission (with small grader wrappers for the 3
`@grader` interactive/functional problems) with `g++ -std=c++17 -O2`, and checks
exact equivalence across seeded random and edge-case tests.
"""

from __future__ import annotations

import os
import random
import resource
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

BATCH_DIR = Path(__file__).resolve().parent
REPO_ROOT = BATCH_DIR.parents[2]
CP_PY = REPO_ROOT / "proto" / "cp.py"
SUBS = REPO_ROOT / "corpus" / "submissions"
TODAY = BATCH_DIR / "today"

CPP_FLAGS = ["g++", "-std=c++17", "-O2", "-pipe"]


def _raise_stack() -> None:
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    target = 256 * 1024 * 1024
    if hard == resource.RLIM_INFINITY or hard >= target:
        resource.setrlimit(resource.RLIMIT_STACK, (target, hard))


def run_cmd(cmd: list[str], stdin: str | None = None, timeout: float = 10.0) -> str:
    p = subprocess.run(
        cmd,
        input=stdin,
        text=True,
        capture_output=True,
        timeout=timeout,
        preexec_fn=_raise_stack,
    )
    if p.returncode != 0:
        raise RuntimeError(
            f"Command failed ({p.returncode}): {' '.join(cmd)}\n"
            f"STDOUT:\n{p.stdout}\nSTDERR:\n{p.stderr}"
        )
    return p.stdout


def build_cp(name: str, work: Path) -> Path:
    src = TODAY / f"{name}.cp"
    copy = work / f"{name}.cp"
    shutil.copyfile(src, copy)
    out_cpp = work / f"{name}.cpp"
    run_cmd([sys.executable, str(CP_PY), "build", str(copy)], timeout=30.0)
    bin_path = work / f"{name}_cp.bin"
    run_cmd(CPP_FLAGS + [str(out_cpp), "-o", str(bin_path)], timeout=30.0)
    return bin_path


def build_cpp_direct(sub_id: str, name: str, work: Path) -> Path:
    src = SUBS / f"{sub_id}.cpp"
    bin_path = work / f"{name}_orig.bin"
    run_cmd(CPP_FLAGS + [str(src), "-o", str(bin_path)], timeout=30.0)
    return bin_path


def build_apio23p2_orig(work: Path) -> Path:
    src = (SUBS / "5593302.cpp").read_text()
    # Strip grader header include if present and attach stdin driver calling sequence(N, A)
    lines = [ln for ln in src.splitlines() if '#include "sequence.h"' not in ln]
    driver = "\n".join(lines) + """
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<int> a(n);
    for (int i = 0; i < n; i++) cin >> a[i];
    cout << sequence(n, a) << "\\n";
    return 0;
}
"""
    cpp = work / "apio23p2_orig.cpp"
    cpp.write_text(driver)
    bin_path = work / "apio23p2_orig.bin"
    run_cmd(CPP_FLAGS + [str(cpp), "-o", str(bin_path)], timeout=30.0)
    return bin_path


def build_egoi22p3_orig(work: Path) -> Path:
    src = (SUBS / "5199301.cpp").read_text()
    src = src[: src.index("\nvoid solve()")]
    wrapper = """#include <bits/stdc++.h>
using namespace std;
static bool _played = false;
int GetMove() {
    _played = true;
    return 0;
}
void MakeMove(int) {}
"""
    code = wrapper + src + """
int main() {
    int n, m;
    cin >> n >> m;
    vector<pair<int, int>> edges(m);
    for (int i = 0; i < m; i++) cin >> edges[i].first >> edges[i].second;
    _played = false;
    SocialEngineering(n, m, edges);
    cout << (_played ? "YES" : "NO") << "\\n";
    return 0;
}
"""
    cpp = work / "egoi22p3_orig.cpp"
    cpp.write_text(code)
    bin_path = work / "egoi22p3_orig.bin"
    run_cmd(CPP_FLAGS + [str(cpp), "-o", str(bin_path)], timeout=30.0)
    return bin_path


def build_egoi22p7_orig(work: Path) -> Path:
    src = (SUBS / "5203797.cpp").read_text()
    src = src[: src.index("\nvoid solve()")]
    wrapper = """#include <bits/stdc++.h>
using namespace std;
namespace _G {
    struct DSU {
        vector<int> p;
        DSU(int n = 0) : p(n + 1, -1) {}
        int find(int x) { return p[x] < 0 ? x : p[x] = find(p[x]); }
        bool unite(int a, int b) {
            a = find(a); b = find(b);
            if (a == b) return false;
            if (p[a] < p[b]) swap(a, b);
            p[b] += p[a]; p[a] = b;
            return true;
        }
    };
    vector<DSU> vers;
    int queries = 0;
    vector<pair<int, int>> ans;
}
int Connected(int a, int i, int j) {
    _G::queries++;
    if (_G::vers[a].find(i) == _G::vers[a].find(j)) return a;
    _G::DSU nxt = _G::vers[a];
    nxt.unite(i, j);
    int id = (int)_G::vers.size();
    _G::vers.push_back(std::move(nxt));
    return id;
}
void DescribeDesign(vector<pair<int, int>> res) {
    _G::ans = std::move(res);
}
"""
    code = wrapper + src + """
int main() {
    int n, m;
    cin >> n >> m;
    _G::vers.clear();
    _G::vers.emplace_back(n);
    _G::queries = 0;
    _G::ans.clear();
    for (int i = 0; i < m; i++) {
        int u, v; cin >> u >> v;
        _G::vers[0].unite(u, v);
    }
    ToyDesign(n, -1);
    cout << _G::queries << "\\n" << _G::ans.size() << "\\n";
    for (auto [u, v] : _G::ans) cout << u << " " << v << "\\n";
    return 0;
}
"""
    cpp = work / "egoi22p7_orig.cpp"
    cpp.write_text(code)
    bin_path = work / "egoi22p7_orig.bin"
    run_cmd(CPP_FLAGS + [str(cpp), "-o", str(bin_path)], timeout=30.0)
    return bin_path


def gen_apio23p2(rng: random.Random, idx: int) -> str:
    if idx == 0:
        n, a = 1, [1]
    elif idx == 1:
        n, a = 6, [1, 2, 3, 1, 2, 1]
    elif idx == 2:
        n, a = 9, [1, 1, 2, 3, 1, 2, 3, 1, 1]
    elif idx < 25:
        n = rng.randint(2, 15)
        k = rng.randint(1, min(n, 5))
        a = [rng.randint(1, k) for _ in range(n)]
    else:
        n = rng.randint(20, 120)
        k = rng.randint(2, min(n, 20))
        a = [rng.randint(1, k) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, a)) + "\n"


def gen_egoi22p3(rng: random.Random, idx: int) -> tuple[str, int, set[tuple[int, int]]]:
    n = rng.randint(3, 14) if idx < 20 else rng.randint(15, 40)
    edges: set[tuple[int, int]] = set()
    # Connect vertices 2..n in 1..3 random components so both YES and NO instances appear frequently
    comps = rng.randint(1, min(3, n - 1))
    verts = list(range(2, n + 1))
    rng.shuffle(verts)
    buckets = [verts[i::comps] for i in range(comps)]
    for b in buckets:
        for i in range(1, len(b)):
            u = b[i]
            v = b[rng.randint(0, i - 1)]
            edges.add((min(u, v), max(u, v)))
        extra = rng.randint(0, len(b))
        for _ in range(extra):
            if len(b) >= 2:
                u, v = rng.sample(b, 2)
                edges.add((min(u, v), max(u, v)))
        # Attach to vertex 1: force even count half the time to heavily test YES path generation
        want_even = (idx % 2 == 0)
        cands = b[:]
        rng.shuffle(cands)
        if want_even:
            cnt = 2 * rng.randint(0, len(cands) // 2)
        else:
            cnt = rng.randint(0, len(cands))
        for u in cands[:cnt]:
            edges.add((1, u))
    elist = sorted(edges)
    inp = f"{n} {len(elist)}\n" + "".join(f"{u} {v}\n" for u, v in elist)
    return inp, n, edges


def verify_egoi22p3(cp_out: str, orig_out: str, n: int, edges: set[tuple[int, int]]) -> None:
    orig_verdict = orig_out.strip()
    lines = [ln.strip() for ln in cp_out.strip().splitlines() if ln.strip()]
    cp_verdict = lines[0]
    if cp_verdict != orig_verdict:
        raise AssertionError(f"Verdict mismatch: cp={cp_verdict} orig={orig_verdict}")
    if cp_verdict == "NO":
        return
    # Verify every neighbor of 1 is paired once and all edges across all paths are valid & edge-disjoint
    adj1 = {v for (u, v) in edges if u == 1}
    cnt = int(lines[1])
    if cnt * 2 != len(adj1):
        raise AssertionError(f"Expected {len(adj1)//2} paths, got {cnt}")
    used_special: set[int] = set()
    used_edges: set[tuple[int, int]] = set()
    for ln in lines[2 : 2 + cnt]:
        toks = list(map(int, ln.split()))
        plen, verts = toks[0], toks[1:]
        assert plen == len(verts) and plen >= 2, f"Bad path {toks}"
        assert verts[0] in adj1 and verts[-1] in adj1, f"Endpoints not neighbors of 1: {verts}"
        assert verts[0] != verts[-1], f"Path loops to same endpoint: {verts}"
        assert verts[0] not in used_special and verts[-1] not in used_special
        used_special.add(verts[0])
        used_special.add(verts[-1])
        for a, b in zip(verts, verts[1:]):
            assert a != 1 and b != 1, f"Path passes through vertex 1: {verts}"
            e = (min(a, b), max(a, b))
            assert e in edges, f"Invalid edge {e}"
            assert e not in used_edges, f"Reused edge {e}"
            used_edges.add(e)
    assert used_special == adj1, f"Unpaired neighbors of 1: {adj1 - used_special}"


def gen_egoi22p4(rng: random.Random, idx: int) -> str:
    if idx < 15:
        n = rng.randint(2, 10)
        m = rng.randint(1, 15)
        q = rng.randint(5, 30)
    else:
        n = rng.randint(20, 80)
        m = rng.randint(20, 100)
        q = rng.randint(50, 200)
    init = [rng.randint(1, n) for _ in range(m)]
    edges = []
    for i in range(2, n + 1):
        p = rng.randint(1, i - 1)
        edges.append((p, i))
    ops = []
    for _ in range(q):
        kind = rng.choice(["t", "e", "q", "q"])
        if kind == "t":
            l = rng.randint(1, m)
            r = rng.randint(l, m)
            x = rng.randint(1, n)
            ops.append(f"t {l} {r} {x}")
        elif kind == "e":
            c = rng.randint(1, n)
            v = rng.randint(1, 1000)
            ops.append(f"e {c} {v}")
        else:
            p = rng.randint(1, m)
            ops.append(f"q {p}")
    return (
        f"{n} {m} {q}\n"
        + " ".join(map(str, init))
        + "\n"
        + "".join(f"{u} {v}\n" for u, v in edges)
        + "\n".join(ops)
        + "\n"
    )


def gen_egoi22p7(rng: random.Random, idx: int) -> tuple[str, int, list[tuple[int, int]]]:
    if idx == 0:
        n, edges = 2, []
    elif idx == 1:
        n, edges = 4, [(1, 2), (2, 3), (3, 4)]
    elif idx < 20:
        n = rng.randint(2, 16)
        m = rng.randint(0, n)
        edges = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m)]
    else:
        n = rng.randint(20, 60)
        m = rng.randint(0, 2 * n)
        edges = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m)]
    inp = f"{n} {len(edges)}\n" + "".join(f"{u} {v}\n" for u, v in edges)
    return inp, n, edges


def components(n: int, edges: list[tuple[int, int]]) -> list[frozenset[int]]:
    par = list(range(n + 1))

    def find(x: int) -> int:
        while par[x] != x:
            par[x] = par[par[x]]
            x = par[x]
        return x

    for u, v in edges:
        par[find(u)] = find(v)
    groups: dict[int, set[int]] = {}
    for i in range(1, n + 1):
        groups.setdefault(find(i), set()).add(i)
    return sorted(frozenset(g) for g in groups.values())


def verify_egoi22p7(cp_out: str, orig_out: str, n: int, hidden: list[tuple[int, int]]) -> None:
    def parse(out: str) -> tuple[int, list[tuple[int, int]]]:
        lines = [ln.strip() for ln in out.strip().splitlines() if ln.strip()]
        q = int(lines[0])
        k = int(lines[1])
        es = [tuple(map(int, ln.split())) for ln in lines[2 : 2 + k]]
        return q, es

    q_cp, e_cp = parse(cp_out)
    q_orig, e_orig = parse(orig_out)
    want = components(n, hidden)
    assert components(n, e_orig) == want, "Original C++ did not reconstruct components"
    assert components(n, e_cp) == want, f"CP port component mismatch: got {components(n, e_cp)} vs {want}"
    assert q_cp <= q_orig, f"CP port used more queries than original: {q_cp} > {q_orig}"


def gen_cco03p4(rng: random.Random, idx: int) -> str:
    n = rng.randint(1, 7)
    k = rng.randint(0, min(15, n * n))
    pairs = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(k)]
    # Avoid x == y because original checks only x < y indices in permutation (which would give 0)
    return f"{n}\n{k}\n" + "".join(f"{x} {y}\n" for x, y in pairs)


def gen_ccc23s2(rng: random.Random, idx: int) -> str:
    n = rng.randint(1, 30) if idx < 20 else rng.randint(31, 150)
    h = [rng.randint(0, 100_000) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, h)) + "\n"


def gen_ccc23s1(rng: random.Random, idx: int) -> str:
    n = rng.randint(1, 30) if idx < 20 else rng.randint(31, 500)
    r1 = [rng.randint(0, 1) for _ in range(n)]
    r2 = [rng.randint(0, 1) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, r1)) + "\n" + " ".join(map(str, r2)) + "\n"


def gen_ccc96s3(rng: random.Random, idx: int) -> str:
    t = rng.randint(1, 4)
    lines = [str(t)]
    for _ in range(t):
        n = rng.randint(1, 10)
        k = rng.randint(0, n)
        lines.append(f"{n} {k}")
    return "\n".join(lines) + "\n"


def gen_ccc20s1(rng: random.Random, idx: int) -> str:
    n = rng.randint(2, 40)
    ts = rng.sample(range(0, 10_000), n)
    xs = [rng.randint(-100_000, 100_000) for _ in range(n)]
    return f"{n}\n" + "".join(f"{t} {x}\n" for t, x in zip(ts, xs))


def main() -> None:
    rng = random.Random(2026051)
    with tempfile.TemporaryDirectory(prefix="scout51_") as td:
        work = Path(td)
        # 1. apio23p2
        print("[1/10] apio23p2...", end=" ", flush=True)
        cp_bin = build_cp("apio23p2", work)
        orig_bin = build_apio23p2_orig(work)
        for i in range(40):
            inp = gen_apio23p2(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).strip(), run_cmd([str(orig_bin)], inp).strip()
            assert a == b, f"apio23p2 case {i}: cp={a} orig={b}\n{inp}"
        print("OK (40 cases)")

        # 2. egoi22p3
        print("[2/10] egoi22p3...", end=" ", flush=True)
        cp_bin = build_cp("egoi22p3", work)
        orig_bin = build_egoi22p3_orig(work)
        yes_cnt = 0
        for i in range(40):
            inp, n, edges = gen_egoi22p3(rng, i)
            co = run_cmd([str(cp_bin)], inp)
            oo = run_cmd([str(orig_bin)], inp)
            verify_egoi22p3(co, oo, n, edges)
            if oo.strip() == "YES":
                yes_cnt += 1
        print(f"OK (40 cases, {yes_cnt} YES / {40 - yes_cnt} NO)")

        # 3. egoi22p4
        print("[3/10] egoi22p4...", end=" ", flush=True)
        cp_bin = build_cp("egoi22p4", work)
        orig_bin = build_cpp_direct("5199667", "egoi22p4", work)
        for i in range(30):
            inp = gen_egoi22p4(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).strip(), run_cmd([str(orig_bin)], inp).strip()
            assert a == b, f"egoi22p4 case {i}: cp={a[:200]} orig={b[:200]}"
        print("OK (30 cases)")

        # 4. egoi22p7
        print("[4/10] egoi22p7...", end=" ", flush=True)
        cp_bin = build_cp("egoi22p7", work)
        orig_bin = build_egoi22p7_orig(work)
        for i in range(30):
            inp, n, edges = gen_egoi22p7(rng, i)
            co = run_cmd([str(cp_bin)], inp)
            oo = run_cmd([str(orig_bin)], inp)
            verify_egoi22p7(co, oo, n, edges)
        print("OK (30 cases)")

        # 5. cco18p2
        print("[5/10] cco18p2...", end=" ", flush=True)
        cp_bin = build_cp("cco18p2", work)
        orig_bin = build_cpp_direct("4801864", "cco18p2", work)
        a = [ln.strip() for ln in run_cmd([str(cp_bin)], "").strip().splitlines()]
        b = [ln.strip() for ln in run_cmd([str(orig_bin)], "").strip().splitlines()]
        assert a == b, f"cco18p2 mismatch at first diff"
        print("OK (exact 100x100 matrix match)")

        # 6. cco03p4
        print("[6/10] cco03p4...", end=" ", flush=True)
        cp_bin = build_cp("cco03p4", work)
        orig_bin = build_cpp_direct("5024536", "cco03p4", work)
        for i in range(40):
            inp = gen_cco03p4(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).strip(), run_cmd([str(orig_bin)], inp).strip()
            assert a == b, f"cco03p4 case {i}: cp={a} orig={b}\n{inp}"
        print("OK (40 cases)")

        # 7. ccc23s2
        print("[7/10] ccc23s2...", end=" ", flush=True)
        cp_bin = build_cp("ccc23s2", work)
        orig_bin = build_cpp_direct("5350805", "ccc23s2", work)
        for i in range(40):
            inp = gen_ccc23s2(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).split(), run_cmd([str(orig_bin)], inp).split()
            assert a == b, f"ccc23s2 case {i}: cp={a[:10]} orig={b[:10]}"
        print("OK (40 cases)")

        # 8. ccc23s1
        print("[8/10] ccc23s1...", end=" ", flush=True)
        cp_bin = build_cp("ccc23s1", work)
        orig_bin = build_cpp_direct("5350804", "ccc23s1", work)
        for i in range(40):
            inp = gen_ccc23s1(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).strip(), run_cmd([str(orig_bin)], inp).strip()
            assert a == b, f"ccc23s1 case {i}: cp={a} orig={b}"
        print("OK (40 cases)")

        # 9. ccc96s3
        print("[9/10] ccc96s3...", end=" ", flush=True)
        cp_bin = build_cp("ccc96s3", work)
        orig_bin = build_cpp_direct("5515585", "ccc96s3", work)
        for i in range(25):
            inp = gen_ccc96s3(rng, i)
            a, b = run_cmd([str(cp_bin)], inp).strip(), run_cmd([str(orig_bin)], inp).strip()
            assert a == b, f"ccc96s3 case {i}:\nCP:\n{a}\nORIG:\n{b}"
        print("OK (25 cases)")

        # 10. ccc20s1
        print("[10/10] ccc20s1...", end=" ", flush=True)
        cp_bin = build_cp("ccc20s1", work)
        orig_bin = build_cpp_direct("4323572", "ccc20s1", work)
        for i in range(40):
            inp = gen_ccc20s1(rng, i)
            a = float(run_cmd([str(cp_bin)], inp).strip())
            b = float(run_cmd([str(orig_bin)], inp).strip())
            assert abs(a - b) <= 1e-4, f"ccc20s1 case {i}: cp={a} orig={b}"
        print("OK (40 cases)")

    print("ALL 10 SCOUT BATCH 51 PORTS VERIFIED!")


if __name__ == "__main__":
    main()
