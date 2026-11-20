#!/usr/bin/env python3
"""Differential checker for Scout Batch 38 (Hard Olympiad Classics).

Compiles each original C++ submission in corpus/submissions/ and each runnable
port in today/ (via proto/cpc for 9 ports, and proto/cp.py lib-run for dpf),
then tests both on small/medium random inputs and edge cases (with semantic
validators for multi-answer problems). Never stamps tracked .cp files.
"""
import os
import random
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
TODAY = os.path.join(os.path.dirname(__file__), "today")
SUBS = os.path.join(ROOT, "corpus", "submissions")
CP_PY = os.path.join(ROOT, "proto", "cp.py")


def compile_cpp(src: str, out: str) -> None:
    r = subprocess.run(
        ["g++", "-std=c++17", "-O2", src, "-o", out],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(f"g++ failed on {src}:\n{r.stderr}")


def compile_cp(cp_src: str, td: str, name: str) -> str:
    tmp_cp = os.path.join(td, f"{name}.cp")
    shutil.copy2(cp_src, tmp_cp)
    r = subprocess.run(["python3", CP_PY, "build", tmp_cp], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"cpc build failed on {cp_src}:\n{r.stdout}\n{r.stderr}")
    cpp = os.path.join(td, f"{name}.cpp")
    exe = os.path.join(td, f"{name}_cp")
    compile_cpp(cpp, exe)
    return exe


def run_bin(cmd, inp: str, timeout: float = 10.0) -> str:
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"cmd {cmd} failed (rc={r.returncode}):\n{r.stderr}\ninput:\n{inp[:400]}")
    return r.stdout


def tokens(s: str):
    return s.split()


# --- Generators & Validators ---

def brute_apio09p1(n, m, k, grid):
    # Exact brute force over all 3 non-overlapping k x k squares for small grids.
    sq = []
    for i in range(n - k + 1):
        for j in range(m - k + 1):
            s = sum(grid[x][y] for x in range(i, i + k) for y in range(j, j + k))
            sq.append((i, j, s))
    best = 0
    L = len(sq)
    for a in range(L):
        i1, j1, s1 = sq[a]
        for b in range(a + 1, L):
            i2, j2, s2 = sq[b]
            if abs(i1 - i2) < k and abs(j1 - j2) < k:
                continue
            for c in range(b + 1, L):
                i3, j3, s3 = sq[c]
                if (abs(i1 - i3) < k and abs(j1 - j3) < k) or (abs(i2 - i3) < k and abs(j2 - j3) < k):
                    continue
                if s1 + s2 + s3 > best:
                    best = s1 + s2 + s3
    return best


def test_apio09p1(cpp_exe, cp_exe, rng):
    # Note: original 5536569.cpp has an OOB read `ans[n]` in solve_max_down when k == 1
    # (`vector<int> ans(n); ans[n-1] = ans[n];`). So we check k == 1 against exact brute
    # force, and k >= 2 against both brute force (small) and 5536569.cpp (all).
    n_cases = 0
    for _ in range(15):
        k = rng.randint(1, 2)
        n = rng.randint(3 * k, 3 * k + 3)
        m = rng.randint(3 * k, 3 * k + 3)
        grid = [[rng.randint(0, 50) for _ in range(m)] for _ in range(n)]
        inp = f"{n} {m} {k}\n" + "\n".join(" ".join(map(str, row)) for row in grid) + "\n"
        want = brute_apio09p1(n, m, k, grid)
        got = int(tokens(run_bin([cp_exe], inp))[0])
        assert got == want, f"apio09p1 brute case {n_cases}: want={want} cp={got}"
        if k >= 2:
            cpp_got = int(tokens(run_bin([cpp_exe], inp))[0])
            assert cpp_got == want, f"apio09p1 orig mismatch {n_cases}: want={want} cpp={cpp_got}"
        n_cases += 1
    for _ in range(25):
        k = rng.randint(2, 5)
        n = rng.randint(3 * k, 3 * k + 10)
        m = rng.randint(3 * k, 3 * k + 10)
        lines = [f"{n} {m} {k}"]
        for _ in range(n):
            lines.append(" ".join(str(rng.randint(0, 50)) for _ in range(m)))
        inp = "\n".join(lines) + "\n"
        a = tokens(run_bin([cpp_exe], inp))
        b = tokens(run_bin([cp_exe], inp))
        assert a == b, f"apio09p1 diff case {n_cases}: cpp={a} cp={b}"
        n_cases += 1
    # medium case
    k, n, m = 7, 45, 40
    lines = [f"{n} {m} {k}"] + [" ".join(str(rng.randint(0, 1000)) for _ in range(m)) for _ in range(n)]
    inp = "\n".join(lines) + "\n"
    assert tokens(run_bin([cpp_exe], inp)) == tokens(run_bin([cp_exe], inp))
    return n_cases + 1


def test_apio09p2(cpp_exe, cp_exe, rng):
    cases = []
    for t in range(40):
        n = rng.randint(1, 30) if t < 35 else rng.randint(100, 300)
        hi = rng.choice([10, 30, 1000])
        lines = [str(n)]
        for _ in range(n):
            l = rng.randint(1, hi)
            r = rng.randint(l, hi)
            lines.append(f"{l} {r}")
        cases.append("\n".join(lines) + "\n")
    for idx, inp in enumerate(cases):
        a = tokens(run_bin([cpp_exe], inp))
        b = tokens(run_bin([cp_exe], inp))
        assert a == b, f"apio09p2 case {idx}: cpp={a} cp={b}\ninp:\n{inp[:300]}"
    return len(cases)


def test_btoi04p3(cpp_exe, cp_exe, rng):
    cases = []
    cases.append("7\n9\n4\n8\n20\n14\n15\n18\n")
    cases.append("5\n5\n4\n3\n2\n1\n")
    cases.append("5\n1\n2\n3\n4\n5\n")
    cases.append("1\n42\n")
    for t in range(40):
        n = rng.randint(1, 50) if t < 35 else rng.randint(500, 1500)
        hi = rng.choice([5, 30, 10000])
        arr = [rng.randint(0, hi) for _ in range(n)]
        cases.append(f"{n}\n" + "\n".join(map(str, arr)) + "\n")
    for idx, inp in enumerate(cases):
        nums = list(map(int, inp.split()))
        n, a = nums[0], nums[1:]
        co = list(map(int, run_bin([cpp_exe], inp).split()))
        po = list(map(int, run_bin([cp_exe], inp).split()))
        assert len(po) == n + 1, f"btoi04p3 case {idx}: bad len {len(po)}"
        assert co[0] == po[0], f"btoi04p3 case {idx}: cost cpp={co[0]} cp={po[0]}"
        b = po[1:]
        for i in range(n - 1):
            assert b[i] < b[i + 1], f"btoi04p3 case {idx}: not strictly inc at {i}: {b}"
        cost = sum(abs(a[i] - b[i]) for i in range(n))
        assert cost == po[0], f"btoi04p3 case {idx}: reported {po[0]} != actual {cost}"
    return len(cases)


def test_btoi08p6(cpp_exe, cp_exe, rng):
    cases = []
    for _ in range(40):
        n = rng.randint(1, 12)
        a = [rng.randint(0, 50) for _ in range(n)]
        b = [rng.randint(0, 50) for _ in range(n)]
        k = rng.randrange(n)
        a[k] = max(a[k], 1)
        b[k] = max(b[k], 1)
        cases.append(f"{n}\n" + " ".join(map(str, a)) + "\n" + " ".join(map(str, b)) + "\n")
    for idx, inp in enumerate(cases):
        co = list(map(int, run_bin([cpp_exe], inp).split()))
        po = list(map(int, run_bin([cp_exe], inp).split()))
        assert co == po, f"btoi08p6 case {idx}: cpp={co} cp={po}\ninp:\n{inp}"
    return len(cases)


def test_egoi23p2(cpp_exe, cp_exe, rng):
    cases = []
    for t in range(40):
        n = rng.randint(2, 20) if t < 35 else 200
        m = rng.randint(1, 50) if t < 35 else 500
        lines = [f"{n} {m}"]
        for _ in range(m):
            u = rng.randrange(n)
            v = rng.randrange(n - 1)
            if v >= u:
                v += 1
            lines.append(f"{u} {v}")
        cases.append("\n".join(lines) + "\n")
    for idx, inp in enumerate(cases):
        co = tokens(run_bin([cpp_exe], inp))
        po = tokens(run_bin([cp_exe], inp))
        assert co == po, f"egoi23p2 case {idx}: cpp={co[:20]} cp={po[:20]}"
    return len(cases)


def test_btoi17p4(cpp_exe, cp_exe, rng):
    cases = []
    for t in range(40):
        if t % 2 == 0:
            # Planted valid partitionable graph
            p = rng.randint(2, 5)
            q = rng.randint(1, 4)
            gcnt = rng.randint(1, 7)
            groups = []
            cur = 0
            for _ in range(gcnt):
                sz = rng.randint(1, p)
                groups.append(list(range(cur, cur + sz)))
                cur += sz
            n = cur
            adj = [set() for _ in range(n)]
            for grp in groups:
                for u in grp:
                    for v in grp:
                        if u != v and rng.random() < 0.6:
                            adj[u].add(v)
                            adj[v].add(u)
            # add sparse cross edges respecting q
            cut = [0] * gcnt
            for _ in range(gcnt * q):
                g1 = rng.randrange(gcnt)
                g2 = rng.randrange(gcnt)
                if g1 == g2:
                    continue
                if cut[g1] < q and cut[g2] < q:
                    u = rng.choice(groups[g1])
                    v = rng.choice(groups[g2])
                    if v not in adj[u]:
                        adj[u].add(v)
                        adj[v].add(u)
                        cut[g1] += 1
                        cut[g2] += 1
        else:
            # Random graph (sometimes asymmetric, sometimes dense)
            n = rng.randint(2, 14)
            p = rng.randint(1, 4)
            q = rng.randint(0, 3)
            adj = [set() for _ in range(n)]
            sym = rng.random() < 0.8
            prob = rng.uniform(0.1, 0.5)
            for u in range(n):
                for v in range(u + 1, n):
                    if rng.random() < prob:
                        adj[u].add(v)
                        if sym or rng.random() < 0.5:
                            adj[v].add(u)
        lines = [f"{n} {p} {q}"]
        for u in range(n):
            ns = sorted(adj[u])
            lines.append(" ".join([str(len(ns))] + [str(v) for v in ns]))
        cases.append(("\n".join(lines) + "\n", n, p, q, adj))

    for idx, (inp, n, p, q, adj) in enumerate(cases):
        co = tokens(run_bin([cpp_exe], inp))
        po = tokens(run_bin([cp_exe], inp))
        assert co[0] == po[0], f"btoi17p4 case {idx}: cpp={co[0]} cp={po[0]}"
        if po[0] == "home":
            k = int(po[1])
            pos = 2
            seen = set()
            for _ in range(k):
                sz = int(po[pos]); pos += 1
                grp = set(int(x) for x in po[pos:pos + sz]); pos += sz
                assert 1 <= len(grp) <= p
                assert not (seen & grp)
                seen |= grp
                cut = sum(1 for u in grp for v in adj[u] if v not in grp)
                assert cut <= q, f"btoi17p4 case {idx}: cut {cut} > {q}"
            assert seen == set(range(n)) and pos == len(po)
    return len(cases)


def test_btoi18p4(cpp_exe, cp_exe, rng):
    cases = []
    for t in range(50):
        n = rng.randint(2, 20) if t < 45 else rng.randint(60, 120)
        m = rng.randint(1, 18) if t < 45 else rng.randint(60, 120)
        arcs = []
        for _ in range(m):
            l = rng.randint(1, n)
            r = rng.randint(1, n)
            arcs.append((l, r))
        inp = f"{n} {m}\n" + "\n".join(f"{l} {r}" for l, r in arcs) + "\n"
        cases.append((inp, n, m, arcs))

    for idx, (inp, n, m, arcs) in enumerate(cases):
        co = tokens(run_bin([cpp_exe], inp))
        po = tokens(run_bin([cp_exe], inp))
        assert (co[0] == "impossible") == (po[0] == "impossible"), (
            f"btoi18p4 case {idx}: cpp={co[0]} cp={po[0]}\ninp:\n{inp}"
        )
        if po[0] != "impossible":
            s = po[0]
            assert len(s) == m and set(s) <= {"0", "1"}
            for bit in ("0", "1"):
                cov = [False] * n
                for i, (l, r) in enumerate(arcs):
                    if s[i] != bit:
                        continue
                    x = l - 1
                    while True:
                        cov[x] = True
                        if x == r - 1:
                            break
                        x = (x + 1) % n
                assert all(cov), f"btoi18p4 case {idx}: color {bit} missed segments"
    return len(cases)


def test_egoi23p4(cpp_exe, cp_exe, rng):
    def fw_mn(mat, n):
        g = [row[:] for row in mat]
        for i in range(n):
            g[i][i] = 10**9
        for mid in range(n):
            for u in range(n):
                for v in range(n):
                    val = min(g[u][mid], g[mid][v])
                    if val > g[u][v]:
                        g[u][v] = val
        return g

    def fw_mx(mat, n):
        g = [row[:] for row in mat]
        for i in range(n):
            g[i][i] = -1
        for mid in range(n):
            for u in range(n):
                for v in range(n):
                    val = max(g[u][mid], g[mid][v])
                    if val < g[u][v]:
                        g[u][v] = val
        return g

    cases = []
    for t in range(40):
        n = rng.randint(2, 12) if t < 35 else 25
        kv = rng.randint(5, 50)
        if t % 3 != 2:
            # Plant a connected weighted graph and compute exact bottleneck closures
            g_mn = [[-1] * n for _ in range(n)]
            g_mx = [[10**9] * n for _ in range(n)]
            for i in range(1, n):
                p = rng.randrange(i)
                w = rng.randint(0, kv)
                g_mn[i][p] = g_mn[p][i] = max(g_mn[i][p], w)
                g_mx[i][p] = g_mx[p][i] = min(g_mx[i][p], w)
            for _ in range(rng.randint(0, n * 2)):
                u = rng.randrange(n)
                v = rng.randrange(n)
                if u != v:
                    w = rng.randint(0, kv)
                    g_mn[u][v] = g_mn[v][u] = max(g_mn[u][v], w)
                    g_mx[u][v] = g_mx[v][u] = min(g_mx[u][v], w)
            mat_mn = fw_mn(g_mn, n)
            mat_mx = fw_mx(g_mx, n)
        else:
            # Random matrices (usually NO)
            mat_mn = [[10**9 if i == j else 0 for j in range(n)] for i in range(n)]
            mat_mx = [[-1 if i == j else 0 for j in range(n)] for i in range(n)]
            for i in range(n):
                for j in range(i):
                    mat_mn[i][j] = mat_mn[j][i] = rng.randint(0, kv)
                    mat_mx[i][j] = mat_mx[j][i] = rng.randint(0, kv)
        lines = [f"{n} {kv}"]
        for i in range(1, n):
            lines.append(" ".join(str(kv - mat_mx[i][j]) for j in range(i)))
        for i in range(1, n):
            lines.append(" ".join(str(mat_mn[i][j]) for j in range(i)))
        cases.append(("\n".join(lines) + "\n", n, kv, mat_mn, mat_mx))

    for idx, (inp, n, kv, mat_mn, mat_mx) in enumerate(cases):
        co = tokens(run_bin([cpp_exe], inp))
        po = tokens(run_bin([cp_exe], inp))
        assert (co[0] == "NO") == (po[0] == "NO"), f"egoi23p4 case {idx}: cpp={co[0]} cp={po[0]}"
        if po[0] != "NO":
            m = int(po[0])
            assert m <= 2 * (n - 1) and len(po) == 1 + 3 * m
            cg_mn = [[-1] * n for _ in range(n)]
            cg_mx = [[10**9] * n for _ in range(n)]
            for i in range(m):
                u = int(po[1 + 3 * i])
                v = int(po[2 + 3 * i])
                w = int(po[3 + 3 * i])
                assert 0 <= u < n and 0 <= v < n and u != v and 0 <= w <= kv
                cg_mn[u][v] = cg_mn[v][u] = max(cg_mn[u][v], w)
                cg_mx[u][v] = cg_mx[v][u] = min(cg_mx[u][v], w)
            assert fw_mn(cg_mn, n) == mat_mn and fw_mx(cg_mx, n) == mat_mx
    return len(cases)


def is_subseq(sub: str, s: str) -> bool:
    it = iter(s)
    return all(ch in it for ch in sub)


def test_dpf(cpp_exe, cp_src, rng):
    cases = ["axyb\nabyxb\n", "aa\nxayaz\n", "a\nz\n", "abracadabra\navadakedavra\n"]
    for _ in range(30):
        alpha = rng.choice(["ab", "abc", "abcde", "abcdefghij"])
        la = rng.randint(1, 25)
        lb = rng.randint(1, 25)
        a = "".join(rng.choice(alpha) for _ in range(la))
        b = "".join(rng.choice(alpha) for _ in range(lb))
        cases.append(f"{a}\n{b}\n")
    for idx, inp in enumerate(cases):
        a, b = inp.splitlines()
        co = run_bin([cpp_exe], inp).strip("\n")
        po = run_bin(["python3", CP_PY, "lib-run", cp_src], inp).strip("\n")
        assert len(co) == len(po), f"dpf case {idx}: len cpp={len(co)} ({co!r}) cp={len(po)} ({po!r})"
        assert is_subseq(po, a) and is_subseq(po, b), f"dpf case {idx}: {po!r} not subseq of {a!r}, {b!r}"
    return len(cases)


def test_egoi23p7(cpp_exe, cp_exe, rng):
    cases = []
    for t in range(45):
        n = rng.randint(2, 20) if t < 40 else 150
        if t % 3 != 2:
            # Planted rooted arborescence
            perm = list(range(n))
            rng.shuffle(perm)
            tree_edges = []  # (child, parent)
            for i in range(1, n):
                p = perm[rng.randrange(i)]
                tree_edges.append((perm[i], p))
            tree_set = set(tree_edges)
            rng.shuffle(tree_edges)
            m1 = rng.randint(0, len(tree_edges))
            req = tree_edges[:m1]
            forb = []
            m2 = rng.randint(0, min(n * 2, 40))
            for _ in range(m2):
                u = rng.randrange(n)
                v = rng.randrange(n)
                if u != v and (u, v) not in tree_set:
                    forb.append((u, v))
            forb = list(set(forb))
            m2 = len(forb)
        else:
            # Arbitrary random edges (often NO)
            m1 = rng.randint(0, n)
            m2 = rng.randint(0, n * 2)
            req = [(rng.randrange(n), rng.randrange(n)) for _ in range(m1)]
            req = [(u, v) for u, v in set(req) if u != v]
            forb = [(rng.randrange(n), rng.randrange(n)) for _ in range(m2)]
            forb = [(u, v) for u, v in set(forb) if u != v and (u, v) not in set(req)]
            m1, m2 = len(req), len(forb)
        lines = [f"{n} {m1} {m2}"]
        lines += [f"{u} {v}" for u, v in req]
        lines += [f"{u} {v}" for u, v in forb]
        cases.append(("\n".join(lines) + "\n", n, set(req), set(forb)))

    for idx, (inp, n, req, forb) in enumerate(cases):
        co = tokens(run_bin([cpp_exe], inp))
        po = tokens(run_bin([cp_exe], inp))
        assert (co[0] == "NO") == (po[0] == "NO"), f"egoi23p7 case {idx}: cpp={co[0]} cp={po[0]}\ninp:\n{inp}"
        if po[0] != "NO":
            nums = list(map(int, po))
            assert len(nums) == 2 * (n - 1)
            edges = [(nums[2 * i], nums[2 * i + 1]) for i in range(n - 1)]
            eset = set(edges)
            assert req <= eset, f"egoi23p7 case {idx}: missing req edge"
            assert not (forb & eset), f"egoi23p7 case {idx}: used forb edge"
            # Verify rooted tree: each edge is (child, parent), exactly one root with 0 parents, connected
            parent = {}
            children = [[] for _ in range(n)]
            for c, p in edges:
                assert c not in parent, f"egoi23p7 case {idx}: node {c} has 2 parents"
                parent[c] = p
                children[p].append(c)
            roots = [u for u in range(n) if u not in parent]
            assert len(roots) == 1
            seen = set()
            st = [roots[0]]
            while st:
                u = st.pop()
                seen.add(u)
                for c in children[u]:
                    st.append(c)
            assert len(seen) == n, f"egoi23p7 case {idx}: disconnected"
    return len(cases)


PROBLEMS = [
    ("apio09p1", "5536569", "cpc", test_apio09p1),
    ("apio09p2", "5536701", "cpc", test_apio09p2),
    ("btoi04p3", "5270736", "cpc", test_btoi04p3),
    ("btoi08p6", "5325518", "cpc", test_btoi08p6),
    ("btoi17p4", "4523876", "cpc", test_btoi17p4),
    ("btoi18p4", "4656863", "cpc", test_btoi18p4),
    ("dpf",      "4131645", "cpi", test_dpf),
    ("egoi23p2", "5678050", "cpc", test_egoi23p2),
    ("egoi23p4", "5678052", "cpc", test_egoi23p4),
    ("egoi23p7", "5678057", "cpc", test_egoi23p7),
]


def main():
    rng = random.Random(38_2026)
    with tempfile.TemporaryDirectory() as td:
        for name, sub_id, mode, fn in PROBLEMS:
            cpp_src = os.path.join(SUBS, f"{sub_id}.cpp")
            cpp_exe = os.path.join(td, f"{name}_orig")
            compile_cpp(cpp_src, cpp_exe)
            cp_src = os.path.join(TODAY, f"{name}.cp")
            if mode == "cpc":
                cp_target = compile_cp(cp_src, td, name)
            else:
                cp_target = cp_src
            n_cases = fn(cpp_exe, cp_target, rng)
            print(f"[PASS] {name:10s} (orig {sub_id}.cpp, {mode}): {n_cases} cases verified")


if __name__ == "__main__":
    main()
