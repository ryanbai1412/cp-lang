#!/usr/bin/env python3
"""Check the batch-15 scout ports against the original C++ submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both receive the same
small random inputs and must agree. Problems with multiple valid answers
(`coci25c3p3`) check optimal length against the original and validate the witness
against the problem statement.

    python3 proto/ports/scout_batch_15/check.py                # every port, 40 inputs each
    python3 proto/ports/scout_batch_15/check.py cco16p3 100    # one port, 100 inputs
"""
import os, random, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def gen_noi09p4():
    n, m = R(1, 3), R(1, 4)
    rows = [(n, m)]
    for r in range(n):
        for c in range(m):
            score = R(-15, 15)
            k = R(0, min(3, n * m - 1))
            targets = set()
            for _ in range(k):
                x, y = R(0, n - 1), R(0, m - 1)
                if (x, y) != (r, c):
                    targets.add((x, y))
            row = [score, len(targets)]
            for x, y in targets:
                row.extend([x, y])
            rows.append(row)
    return lines(rows)


def gen_dmopc20c7p5():
    n = R(2, 7)
    m = R(1, 10)
    costs = [R(1, 25) for _ in range(n)]
    rows = [(n, m), costs]
    for _ in range(m):
        u = R(1, n - 1)
        v = R(u + 1, n)
        rows.append((u, v, R(1, 40)))
    return lines(rows)


def gen_sdoi17r1p4():
    n = R(1, 4)
    rows = [(n,)]
    for _ in range(n):
        rows.append([R(1, 30) for _ in range(n)])
    for _ in range(n):
        rows.append([R(1, 30) for _ in range(n)])
    return lines(rows)


def gen_noi09p1():
    n = R(2, 8)
    if R(0, 2) > 0:
        perm = list(range(n))
        random.shuffle(perm)
        d = [min((perm[i] - i) % n, (i - perm[i]) % n) for i in range(n)]
    else:
        d = [R(0, n // 2) for _ in range(n)]
    return lines([(n,), d])


def gen_coci25c3p3():
    n = R(2, 6)
    # Vertex 1 is on left side so both original and port assign left = side 0.
    others = list(range(2, 2 * n + 1))
    random.shuffle(others)
    left = [1] + others[:n - 1]
    right = others[n - 1:]
    edges = set()
    for i in range(n):
        edges.add((left[i], right[i]))
    # Make connected while preserving unique matching: left[i] only links to right[j] with j < i.
    for i in range(1, n):
        edges.add((left[i], right[R(0, i - 1)]))
        for j in range(i):
            if R(0, 2) == 0:
                edges.add((left[i], right[j]))
    edge_list = list(edges)
    random.shuffle(edge_list)
    return lines([(n, len(edge_list))] + edge_list)


def gen_cco16p3():
    c = R(1, 5)
    tcs = R(2, 3)
    rows = [(c, tcs)]
    for _ in range(tcs):
        n = R(3, 10)
        # Connected simple undirected graph (per CCO '16 P3 constraints).
        edges = set()
        for v in range(2, n + 1):
            u = R(1, v - 1)
            edges.add((u, v))
        max_extra = n * (n - 1) // 2 - (n - 1)
        extra = R(0, min(max_extra, R(0, 8)))
        for _ in range(extra):
            u, v = sorted(random.sample(range(1, n + 1), 2))
            edges.add((u, v))
        edge_list = list(edges)
        random.shuffle(edge_list)
        rows.append((n, len(edge_list)))
        rows.extend(edge_list)
    return lines(rows)


def gen_cco21p3():
    n = R(2, 9)
    q = R(1, 12)
    adj = [[] for _ in range(n)]
    for v in range(1, n):
        p = R(0, v - 1)
        adj[p].append(v + 1)
        adj[v].append(p + 1)
    rows = [(n, q)]
    for nbrs in adj:
        random.shuffle(nbrs)
        rows.append([len(nbrs)] + nbrs)
    for _ in range(q):
        rows.append((R(0, 3 * n * n),))
    return lines(rows)


def gen_noi21p3():
    k_scc = R(2, 7)
    sizes = [R(1, 3) for _ in range(k_scc)]
    n = sum(sizes)
    perm = list(range(1, n + 1))
    random.shuffle(perm)
    sccs = []
    pos = 0
    for sz in sizes:
        sccs.append(perm[pos:pos + sz])
        pos += sz
    edges = []
    # Make each SCC strongly connected.
    for comp in sccs:
        if len(comp) > 1:
            for i in range(len(comp)):
                edges.append((comp[i], comp[(i + 1) % len(comp)]))
            if len(comp) > 2 and R(0, 1) == 0:
                u, v = random.sample(comp, 2)
                edges.append((u, v))
    # Build a random rooted tree over SCCs + optional ancestor->descendant edges.
    par = [-1] * k_scc
    anc = [set() for _ in range(k_scc)]
    for i in range(1, k_scc):
        p = R(0, i - 1)
        par[i] = p
        anc[i] = anc[p] | {p}
        edges.append((random.choice(sccs[p]), random.choice(sccs[i])))
        for a in anc[p]:
            if R(0, 2) == 0:
                edges.append((random.choice(sccs[a]), random.choice(sccs[i])))
    random.shuffle(edges)
    kv = R(0, 2)
    q = R(2, 6)
    rows = [(n, len(edges), q, kv)]
    rows.extend(edges)
    for _ in range(q):
        s, t = R(1, n), R(1, n)
        row = [s, t]
        for _ in range(kv):
            row.extend([R(1, n), R(1, n)])
        rows.append(row)
    return lines(rows)


def gen_noi16p2():
    t = R(1, 4)
    rows = [(t,)]
    for _ in range(t):
        n = random.choice([1, 2, 3, 4, 5, 6])
        m = random.choice([1, 2, 3, 4, 5, 6])
        cells = [(r, c) for r in range(1, n + 1) for c in range(1, m + 1)]
        k = R(0, min(len(cells), 10))
        chosen = random.sample(cells, k)
        rows.append((n, m, k))
        rows.extend(chosen)
    return lines(rows)


def gen_btoi20p3():
    n = R(2, 7)
    m = R(1, 12)
    q = R(1, 10)
    rows = [(n, m, q)]
    if R(0, 3) == 0:
        # Bipartite graph to exercise the fast path.
        side = [R(0, 1) for _ in range(n)]
        left = [i + 1 for i in range(n) if side[i] == 0] or [1]
        right = [i + 1 for i in range(n) if side[i] == 1] or [n]
        for _ in range(m):
            rows.append((random.choice(left), random.choice(right)))
    else:
        for _ in range(m):
            u, v = random.sample(range(1, n + 1), 2)
            rows.append((u, v))
    for _ in range(q):
        l = R(1, m)
        r = R(l, m)
        rows.append((l, r))
    return lines(rows)


def judge_exact(inp, orig, port):
    if orig.strip() != port.strip():
        return f"mismatch: orig={orig.strip()!r} port={port.strip()!r}"
    return None


def judge_sdoi17r1p4(inp, orig, port):
    try:
        o, p = float(orig.strip()), float(port.strip())
    except ValueError:
        return f"non-float output: orig={orig!r} port={port!r}"
    if abs(o - p) > 2e-6:
        return f"float mismatch: orig={orig.strip()} port={port.strip()}"
    return None


def judge_coci25c3p3(inp, orig, port):
    ol = orig.strip().splitlines()
    pl = port.strip().splitlines()
    if not ol or not pl or ol[0] != pl[0]:
        return f"length mismatch: orig={ol[:1]} port={pl[:1]}"
    k = int(pl[0])
    path = list(map(int, pl[1].split()))
    if len(path) != k or len(set(path)) != k:
        return f"invalid path length/uniqueness: {path}"
    toks = list(map(int, inp.split()))
    n, m = toks[0], toks[1]
    edges = set()
    for e in range(m):
        u, v = toks[2 + 2 * e], toks[3 + 2 * e]
        edges.add((u, v))
        edges.add((v, u))
    for i in range(k - 1):
        if (path[i], path[i + 1]) not in edges:
            return f"missing edge ({path[i]}, {path[i + 1]}) in path {path}"
    return None


def brute_noi09p1(inp):
    toks = list(map(int, inp.split()))
    n = toks[0]
    d = toks[1:1 + n]
    cands = [sorted({(i - d[i]) % n, (i + d[i]) % n}) for i in range(n)]
    used = [False] * n
    cur = [0] * n

    def rec(i):
        if i == n:
            return True
        for v in cands[i]:
            if not used[v]:
                used[v] = True
                cur[i] = v
                if rec(i + 1):
                    return True
                used[v] = False
        return False

    return " ".join(map(str, cur)) if rec(0) else "No Answer"


def judge_noi09p1(inp, orig, port):
    want = brute_noi09p1(inp)
    if port.strip() != want:
        return f"mismatch vs exact oracle: want={want!r} port={port.strip()!r}"
    if orig is not None and orig.strip() != want:
        return f"orig disagrees with oracle: orig={orig.strip()!r} want={want!r}"
    return None


PORTS = [
    ("noi09p4",    "5144326", gen_noi09p4,    judge_exact),
    ("dmopc20c7p5","4370353", gen_dmopc20c7p5,judge_exact),
    ("sdoi17r1p4", "5466798", gen_sdoi17r1p4, judge_sdoi17r1p4),
    ("noi09p1",    "5102817", gen_noi09p1,    judge_noi09p1),
    ("coci25c3p3", "7730378", gen_coci25c3p3, judge_coci25c3p3),
    ("cco16p3",    "5568296", gen_cco16p3,    judge_exact),
    ("cco21p3",    "4552873", gen_cco21p3,    judge_exact),
    ("noi21p3",    "5669954", gen_noi21p3,    judge_exact),
    ("noi16p2",    "4798070", gen_noi16p2,    judge_exact),
    ("btoi20p3",   "4656866", gen_btoi20p3,   judge_exact),
]


def compile_orig(sub_id, tmpdir):
    src = os.path.join(ROOT, "corpus", "submissions", f"{sub_id}.cpp")
    exe = os.path.join(tmpdir, f"orig_{sub_id}")
    r = subprocess.run(
        ["g++", "-O2", "-std=gnu++17", src, "-o", exe],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        raise RuntimeError(f"failed to compile {src}:\n{r.stderr}")
    return exe


def run_port(name, inp):
    cp_path = os.path.join(HERE, f"{name}.cp")
    r = subprocess.run(
        ["python3", os.path.join(ROOT, "proto", "cp.py"), "lib-run", cp_path],
        input=inp, capture_output=True, text=True, timeout=30,
    )
    if r.returncode != 0:
        raise RuntimeError(f"port {name} failed:\n{r.stderr}")
    return r.stdout


def raise_stack():
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    target = min(hard, 256 * 1024 * 1024) if hard != resource.RLIM_INFINITY else 256 * 1024 * 1024
    if soft != resource.RLIM_INFINITY and soft < target:
        resource.setrlimit(resource.RLIMIT_STACK, (target, hard))


def main():
    only = sys.argv[1] if len(sys.argv) > 1 else None
    trials = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    random.seed(1515)
    failed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name, sub_id, gen, judge in PORTS:
            if only and name != only:
                continue
            exe = compile_orig(sub_id, tmp)
            ok = 0
            for t in range(trials):
                inp = gen()
                try:
                    orig = subprocess.run([exe], input=inp, capture_output=True, text=True, check=True, timeout=3, preexec_fn=raise_stack).stdout
                except subprocess.TimeoutExpired:
                    if name == "noi09p1":
                        # Original 5102817.cpp loops forever on unsolvable self-loop cases; judge_noi09p1 uses exact oracle.
                        orig = None
                    else:
                        raise
                try:
                    port = run_port(name, inp)
                except Exception as e:
                    print(f"FAIL {name} trial {t}: {e}\n--- input ---\n{inp}", flush=True)
                    failed += 1
                    break
                err = judge(inp, orig, port)
                if err:
                    print(f"FAIL {name} trial {t}: {err}\n--- input ---\n{inp}", flush=True)
                    failed += 1
                    break
                ok += 1
            else:
                print(f"ok   {name:12s} ({ok}/{trials} random inputs match corpus/{sub_id}.cpp)", flush=True)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
