#!/usr/bin/env python3
"""Differential test harness for scout_batch_39 (AAC, YAC, WAC, SAC, DMPG classics).

Compiles each original AC C++ submission from corpus/submissions/ and each
runnable port in today/ (via `proto/cp.py build` on a temporary copy so tracked
sources are never stamped, then `g++ -std=c++17 -O2 -fsanitize=address,undefined`)
and verifies exact agreement (plus a Tarjan SCC constructive validator for aac4p5)
across 150 small and medium random inputs per problem.

    python3 proto/ports/scout_batch_39/check.py               # all 10 ports, 150 cases
    python3 proto/ports/scout_batch_39/check.py yac7p4 300    # one port, 300 cases
"""
import os, random, resource, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def pick_size(rng, idx, small, medium):
    return rng.randint(*(small if idx % 3 else medium))


def gen_aac4p6(rng, idx):
    n = pick_size(rng, idx, (1, 10), (30, 250))
    kind = rng.randrange(5)
    if kind == 0:
        k = 0
    elif kind == 1:
        k = rng.randint(1, n)
    elif kind == 2:
        k = rng.randint(n + 1, 4 * n)
    else:
        k = rng.randint(0, 10**9)
    mode = rng.randrange(4)
    if mode == 0:
        a = [0] * n
    elif mode == 1:
        a = [rng.randint(1, 5) for _ in range(n)]
    elif mode == 2:
        a = [rng.randint(0, 3) for _ in range(n)]
    else:
        a = [rng.randint(0, 2 * n) for _ in range(n)]
    return f"{n} {k}\n" + " ".join(map(str, a)) + "\n"


def gen_aac4p5(rng, idx):
    n = pick_size(rng, idx, (1, 8), (15, 50))
    x = rng.randint(1, n)
    q = rng.randint(0, min(2 * n, 30))
    m = rng.randint(0, min(n * (n - 1), max(4 * n, 40)))
    req = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(q)]
    return f"{n} {m} {x} {q}\n" + "".join(f"{u} {v}\n" for u, v in req)


def tarjan_scc(n, adj):
    idx_cnt = 0
    index = [-1] * n
    low = [0] * n
    on_stack = [False] * n
    stack = []
    comp = [-1] * n
    ncomps = 0
    sys.setrecursionlimit(max(10000, n * 4))

    def dfs(u):
        nonlocal idx_cnt, ncomps
        index[u] = low[u] = idx_cnt
        idx_cnt += 1
        stack.append(u)
        on_stack[u] = True
        for v in adj[u]:
            if index[v] == -1:
                dfs(v)
                low[u] = min(low[u], low[v])
            elif on_stack[v]:
                low[u] = min(low[u], index[v])
        if low[u] == index[u]:
            while True:
                w = stack.pop()
                on_stack[w] = False
                comp[w] = ncomps
                if w == u:
                    break
            ncomps += 1

    for i in range(n):
        if index[i] == -1:
            dfs(i)
    return ncomps, comp


def verify_aac4p5(inp, want_out, got_out):
    want_toks = want_out.split()
    got_toks = got_out.split()
    if want_toks == ["-1"]:
        return got_toks == ["-1"]
    if got_toks == ["-1"]:
        return False
    it = iter(map(int, inp.split()))
    n, m, x, q = next(it), next(it), next(it), next(it)
    reqs = [(next(it) - 1, next(it) - 1) for _ in range(q)]
    if len(got_toks) != 2 * m:
        return False
    edges = set()
    adj = [[] for _ in range(n)]
    for i in range(m):
        u = int(got_toks[2 * i]) - 1
        v = int(got_toks[2 * i + 1]) - 1
        if not (0 <= u < n and 0 <= v < n and u != v) or (u, v) in edges:
            return False
        edges.add((u, v))
        adj[u].append(v)
    ncomps, comp = tarjan_scc(n, adj)
    if ncomps != x:
        return False
    return all(comp[u] == comp[v] for u, v in reqs)


def gen_yac7p4(rng, idx):
    n = pick_size(rng, idx, (1, 10), (20, 100))
    m = pick_size(rng, idx, (1, 20), (30, 150))
    shape = rng.randrange(4)
    edges = []
    for i in range(2, n + 1):
        if shape == 0:
            p = i - 1
        elif shape == 1:
            p = 1
        elif shape == 2:
            p = rng.randint(max(1, i - 3), i - 1)
        else:
            p = rng.randint(1, i - 1)
        edges.append((p, i) if rng.random() < 0.5 else (i, p))
    rng.shuffle(edges)
    qs = [rng.randint(1, n) for _ in range(m)]
    return f"{n} {m}\n" + "".join(f"{u} {v}\n" for u, v in edges) + " ".join(map(str, qs)) + "\n"


def gen_yac6p6(rng, idx):
    n = pick_size(rng, idx, (1, 8), (12, 35))
    m = pick_size(rng, idx, (1, 8), (12, 35))
    q = pick_size(rng, idx, (1, 15), (25, 80))
    hi = rng.choice([5, 100, 10**6])
    grid = [[rng.randint(0, hi) for _ in range(m)] for _ in range(n)]
    qs = []
    for _ in range(q):
        x1, x2 = sorted([rng.randint(1, n), rng.randint(1, n)])
        y1, y2 = sorted([rng.randint(1, m), rng.randint(1, m)])
        kv = rng.randint(0, 5)
        qs.append(f"{x1} {y1} {x2} {y2} {kv}")
    return f"{n} {m} {q}\n" + "\n".join(" ".join(map(str, r)) for r in grid) + "\n" + "\n".join(qs) + "\n"


def gen_yac7p3(rng, idx):
    n = pick_size(rng, idx, (1, 10), (20, 150))
    kind = rng.randrange(4)
    if kind == 0:
        par = list(range(1, n + 1))
        rng.shuffle(par)
    elif kind == 1:
        # Cycle + tail
        c = rng.randint(1, n)
        par = [((i % c) + 1) for i in range(1, c + 1)]
        for i in range(c + 1, n + 1):
            par.append(i - 1)
    else:
        par = [rng.randint(1, n) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, par)) + "\n"


def gen_wac6p6(rng, idx):
    tcs = pick_size(rng, idx, (1, 4), (6, 15))
    out = [str(tcs)]
    for _ in range(tcs):
        branch = rng.randrange(3)
        if branch == 0:
            a = rng.randint(0, 33)
            b = rng.randint(max(a, 46 - a), (100 - a) // 2)
            c = 100 - a - b
        elif branch == 1:
            a = rng.randint(0, 20)
            b = rng.randint(a, min(45 - a, (100 - a) // 2))
            c = 100 - a - b
        else:
            a = rng.randint(0, 22)
            b = rng.randint(a, min(45 - a, (100 - a) // 2))
            c = 100 - a - b
        counts = [a, b, c]
        rng.shuffle(counts)
        out.append(" ".join(map(str, counts)))
        seq = ["R"] * counts[0] + ["G"] * counts[1] + ["B"] * counts[2]
        rng.shuffle(seq)
        out.extend(seq)
    return "\n".join(out) + "\n"


def gen_sac22cc4p4(rng, idx):
    n = pick_size(rng, idx, (1, 12), (30, 250))
    span = rng.choice([3, 20, 10**6])
    a = [rng.randint(-span, span) for _ in range(n)]
    diff = max(a) - min(a)
    mode = rng.randrange(4)
    if mode == 0:
        ql, qr = 0, rng.randint(0, max(0, diff))
    elif mode == 1:
        ql = rng.randint(0, max(0, diff))
        qr = ql
    elif mode == 2:
        ql, qr = 0, 0
    else:
        x, y = rng.randint(0, diff + 5), rng.randint(0, diff + 5)
        ql, qr = min(x, y), max(x, y)
    return f"{n} {ql} {qr}\n" + " ".join(map(str, a)) + "\n"


def gen_sac22cc5p5(rng, idx):
    n = pick_size(rng, idx, (1, 8), (15, 45))
    m = pick_size(rng, idx, (1, 8), (15, 45))
    k = rng.randint(1, min(n, m))
    alpha = rng.choice(["a", "ab", "abc", "abcdef"])
    rows = ["".join(rng.choices(alpha, k=m)) for _ in range(n)]
    return f"{n} {m} {k}\n" + "\n".join(rows) + "\n"


def gen_dmpg16g2(rng, idx):
    n = pick_size(rng, idx, (1, 8), (10, 22))
    dot_w = rng.choice([1, 4, 10])
    alpha = ".ABCDE"
    weights = [dot_w, 1, 1, 1, 1, 1]
    rows = ["".join(rng.choices(alpha, weights=weights, k=n)) for _ in range(n)]
    return f"{n}\n" + "\n".join(rows) + "\n"


def gen_dmpg18s5(rng, idx):
    n = pick_size(rng, idx, (1, 15), (30, 200))
    q = pick_size(rng, idx, (1, 15), (30, 150))
    hi = rng.choice([20, 120, 2000])
    a = [rng.randint(1, hi) for _ in range(n)]
    lines = [f"{n} {q}", " ".join(map(str, a))]
    for _ in range(q):
        if rng.random() < 0.65:
            l, r = sorted([rng.randint(1, n), rng.randint(1, n)])
            x = rng.choice([1, 2, 3, 6, 12, rng.randint(1, hi)])
            lines.append(f"1 {l} {r} {x}")
        else:
            pos = rng.randint(1, n)
            x = rng.randint(1, hi)
            lines.append(f"2 {pos} {x}")
    return "\n".join(lines) + "\n"


PORTS = {
    "aac4p6": (4227368, gen_aac4p6, None),
    "aac4p5": (4140767, gen_aac4p5, verify_aac4p5),
    "yac7p4": (5705324, gen_yac7p4, None),
    "yac6p6": (5150025, gen_yac6p6, None),
    "yac7p3": (5705218, gen_yac7p3, None),
    "wac6p6": (5122001, gen_wac6p6, None),
    "sac22cc4p4": (4411200, gen_sac22cc4p4, None),
    "sac22cc5p5": (4575631, gen_sac22cc5p5, None),
    "dmpg16g2": (4653887, gen_dmpg16g2, None),
    "dmpg18s5": (3476383, gen_dmpg18s5, None),
}


def raise_stack():
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    resource.setrlimit(resource.RLIMIT_STACK, (hard, hard))


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def build(name, sub, tmp):
    ref = os.path.join(tmp, name + ".ref")
    r = sh(["g++", "-std=c++17", "-O2", "-w", "-o", ref,
            os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp")])
    assert r.returncode == 0, r.stderr
    src = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), src)
    r = sh([sys.executable, CP, "build", src], cwd=os.path.join(ROOT, "proto"))
    assert r.returncode == 0 and os.path.exists(src + "p"), r.stdout + r.stderr
    port = os.path.join(tmp, name + ".port")
    r = sh(["g++", "-std=c++17", "-O2", "-fsanitize=address,undefined",
            "-fno-omit-frame-pointer", "-w", "-o", port, src + "p"])
    assert r.returncode == 0, r.stderr
    return [ref], [port]


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(PORTS)
    cases = int(sys.argv[2]) if len(sys.argv) > 2 else 150
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            sub, gen, checker = PORTS[name]
            ref, port = build(name, sub, tmp)
            rng = random.Random(39)
            failed = None
            for idx in range(cases):
                inp = gen(rng, idx)
                want = sh(ref, input=inp, preexec_fn=raise_stack)
                got = sh(port, input=inp)
                if want.returncode != 0 or got.returncode != 0:
                    failed = (idx, inp, want, got)
                    break
                if checker is not None:
                    ok = checker(inp, want.stdout, got.stdout)
                else:
                    ok = [l.split() for l in want.stdout.strip().splitlines()] == [
                        l.split() for l in got.stdout.strip().splitlines()
                    ]
                if not ok:
                    failed = (idx, inp, want, got)
                    break
            if failed:
                bad += 1
                idx, inp, want, got = failed
                print(f"FAIL {name} case {idx} (exit {want.returncode} vs {got.returncode})")
                print(inp[:600], "--- want", want.stdout[:300], "--- got", got.stdout[:300], got.stderr[:400], sep="\n")
            else:
                print(f"ok   {name}: {cases} cases match corpus/submissions/{sub}.cpp (ASan+UBSan clean)")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
