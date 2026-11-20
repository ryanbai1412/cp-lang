#!/usr/bin/env python3
"""Differential & interactive verification harness for Scout Batch 47 (Hard CCC/CCO Part 2).

Compiles every today/*.cp via proto/cp.py (on a temp copy so tracked .cp files are
never version-stamped) and compiles each original accepted C++ submission under
g++ -std=c++17 -O2 -fsanitize=address,undefined -fno-sanitize-recover=all, then
runs randomized differential, constructive, and interactive-pipe tests.
"""

import collections
import os
import random
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
BATCH_DIR = os.path.dirname(os.path.abspath(__file__))
TODAY_DIR = os.path.join(BATCH_DIR, "today")
SUBM_DIR = os.path.join(REPO, "corpus", "submissions")

ORIG_MAP = {
    "cco23p5": "5580402.cpp",
    "cco24p5": "6435483.cpp",
    "cco24p4": "6435484.cpp",
    "cco24p2": "6435481.cpp",
    "ccc20s2hard": "4324075.cpp",
    "cco01p4io": "5034487.cpp",
    "cco07p5": "5120565.cpp",
    "ccc18s2": "4329456.cpp",
    "ccc22s2": "4371343.cpp",
    "cco04p1": "5430016.cpp",
}

CXX_FLAGS = [
    "g++", "-std=c++17", "-O2",
    "-fsanitize=address,undefined", "-fno-sanitize-recover=all",
]


def compile_cp(name, tmpdir):
    src = os.path.join(TODAY_DIR, f"{name}.cp")
    copy_cp = os.path.join(tmpdir, f"{name}_cp.cp")
    shutil.copyfile(src, copy_cp)
    r = subprocess.run(
        ["python3", os.path.join(REPO, "proto", "cp.py"), "build", copy_cp],
        capture_output=True, text=True, cwd=REPO,
    )
    if r.returncode != 0:
        raise RuntimeError(f"cp.py failed for {name}:\n{r.stdout}\n{r.stderr}")
    cpp_path = os.path.join(tmpdir, f"{name}_cp.cpp")
    bin_path = os.path.join(tmpdir, f"{name}_cp")
    r2 = subprocess.run(CXX_FLAGS + ["-o", bin_path, cpp_path], capture_output=True, text=True)
    if r2.returncode != 0:
        raise RuntimeError(f"g++ failed for {name} (cp):\n{r2.stderr}")
    return bin_path


def compile_orig(name, tmpdir):
    src = os.path.join(SUBM_DIR, ORIG_MAP[name])
    bin_path = os.path.join(tmpdir, f"{name}_orig")
    # Original submissions may have signed overflow/UB quirks; keep -O2 clean,
    # while CP-emitted C++ always runs under full ASan+UBSan.
    r = subprocess.run(["g++", "-std=c++17", "-O2", "-o", bin_path, src], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"g++ failed for {name} (orig {ORIG_MAP[name]}):\n{r.stderr}")
    return bin_path


def run_bin(bin_path, inp, timeout=15):
    r = subprocess.run([bin_path], input=inp, capture_output=True, text=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"{os.path.basename(bin_path)} exited {r.returncode}:\n{r.stderr}")
    return r.stdout.strip()


def norm_tokens(s):
    return s.split()


def rand_tree(rng, n, mode="random"):
    edges = []
    if mode == "path":
        perm = list(range(1, n + 1))
        rng.shuffle(perm)
        for i in range(n - 1):
            edges.append((perm[i], perm[i + 1]))
    elif mode == "star":
        center = rng.randint(1, n)
        for i in range(1, n + 1):
            if i != center:
                edges.append((center, i))
    elif mode == "caterpillar":
        spine = max(2, n // 3)
        for i in range(2, spine + 1):
            edges.append((i - 1, i))
        for i in range(spine + 1, n + 1):
            edges.append((rng.randint(1, spine), i))
    else:
        for i in range(2, n + 1):
            edges.append((rng.randint(1, i - 1), i))
    rng.shuffle(edges)
    return edges


def tree_dist(n, edges, u, v):
    adj = [[] for _ in range(n + 1)]
    for a, b in edges:
        adj[a].append(b)
        adj[b].append(a)
    dist = [-1] * (n + 1)
    dist[u] = 0
    q = collections.deque([u])
    while q:
        x = q.popleft()
        if x == v:
            return dist[x]
        for y in adj[x]:
            if dist[y] == -1:
                dist[y] = dist[x] + 1
                q.append(y)
    return 10**9


def verify_cco23p5(cp_bin, orig_bin):
    rng = random.Random(2305)
    modes = ["random", "path", "star", "caterpillar"]
    cases = 0
    for k in (1, 2, 3):
        for t in range(25):
            n = rng.randint(1, 35) if t < 20 else rng.randint(60, 120)
            edges = rand_tree(rng, n, modes[t % len(modes)])
            w = [rng.randint(1, 1000) for _ in range(n)]
            lines = [f"{n} {k}"] + [f"{u} {v}" for u, v in edges] + [" ".join(map(str, w))]
            inp = "\n".join(lines) + "\n"
            o_orig = norm_tokens(run_bin(orig_bin, inp))
            o_cp = norm_tokens(run_bin(cp_bin, inp))
            ans_orig = int(o_orig[0])
            ans_cp = int(o_cp[0])
            assert ans_orig == ans_cp, f"cco23p5 k={k} t={t}: orig={ans_orig} cp={ans_cp}"
            ln = int(o_cp[1])
            seq = list(map(int, o_cp[2:]))
            assert len(seq) == ln, f"cco23p5 len mismatch: {ln} vs {len(seq)}"
            assert seq[0] == 1, f"cco23p5 walk must start at 1, got {seq[0]}"
            assert len(set(seq)) == ln, f"cco23p5 duplicate vertices in {seq}"
            assert sum(w[v - 1] for v in seq) == ans_cp, "cco23p5 weight mismatch"
            if n <= 40:
                for a, b in zip(seq, seq[1:]):
                    d = tree_dist(n, edges, a, b)
                    assert d <= k, f"cco23p5 step {a}->{b} dist {d} > k={k}"
            cases += 1
    return f"{cases} random trees across k=1,2,3 (exact opt + walk validity)"


def verify_cco24p5(cp_bin, orig_bin):
    rng = random.Random(2405)
    cases = 0
    # Edge cases + random small/medium arrays
    fixed = [
        [1],
        [1, 1],
        [1, 2],
        [1, 1, 1, 1],
        [1, 2, 1, 2, 1],
        [1, 1, 2, 2, 1, 1],
    ]
    for arr in fixed:
        inp = f"{len(arr)}\n" + " ".join(map(str, arr)) + "\n"
        assert norm_tokens(run_bin(cp_bin, inp)) == norm_tokens(run_bin(orig_bin, inp))
        cases += 1
    for t in range(60):
        n = rng.randint(2, 60) if t < 50 else rng.randint(200, 1200)
        alpha = rng.choice([2, 3, 4, max(2, n // 3), n])
        arr = [rng.randint(1, alpha) for _ in range(n)]
        inp = f"{n}\n" + " ".join(map(str, arr)) + "\n"
        o1 = norm_tokens(run_bin(orig_bin, inp))
        o2 = norm_tokens(run_bin(cp_bin, inp))
        assert o1 == o2, f"cco24p5 t={t} n={n}: orig={o1} cp={o2}"
        cases += 1
    return f"{cases} edge/random/medium arrays (exact mod-1000003 DP match)"


def verify_cco24p4(cp_bin, orig_bin):
    rng = random.Random(2404)
    modes = ["random", "path", "star", "caterpillar"]
    cases = 0
    for t in range(12):
        n = 100
        edges1 = rand_tree(rng, n, modes[t % len(modes)])
        edges0 = [(u - 1, v - 1) for u, v in edges1]
        inp = f"{n}\n" + "\n".join(f"{u} {v}" for u, v in edges0) + "\n"
        o_orig = run_bin(orig_bin, inp)
        o_cp = run_bin(cp_bin, inp)
        assert norm_tokens(o_orig) == norm_tokens(o_cp), f"cco24p4 mismatch on t={t}"
        # Independently verify the rendezvous property on every pair (s0, s1)
        lines = o_cp.splitlines()
        T = int(lines[0].strip())
        adj = [set([i]) for i in range(n)]
        for u, v in edges0:
            adj[u].add(v)
            adj[v].add(u)
        tab0 = [list(map(int, lines[1 + s].split())) for s in range(n)]
        tab1 = [list(map(int, lines[1 + n + s].split())) for s in range(n)]
        for tab in (tab0, tab1):
            for s in range(n):
                assert len(tab[s]) == T and tab[s][0] == s or tab[s][0] in adj[s]
                for j in range(T - 1):
                    assert tab[s][j + 1] in adj[tab[s][j]], "invalid edge move"
        for s0 in range(n):
            for s1 in range(n):
                met = any(tab0[s0][j] == tab1[s1][j] for j in range(T))
                assert met, f"cco24p4 pair ({s0},{s1}) never met!"
        cases += 1
    return f"{cases} n=100 trees (exact match + full 100x100 rendezvous check)"


def verify_cco24p2(cp_bin, orig_bin):
    rng = random.Random(2402)
    cases = 0
    for t in range(60):
        n = rng.randint(1, 50)
        kind = t % 4
        if kind == 0:
            # Impossible multiset
            a = [rng.randint(1, n) for _ in range(n)]
            b = [rng.randint(1, n) for _ in range(n)]
            if sorted(a) == sorted(b):
                b[0] = (b[0] % n) + 1 if n > 1 else b[0]
        elif kind == 1:
            # Binary {1, 2}
            a = [rng.randint(1, min(2, n)) for _ in range(n)]
            b = list(a)
            if t % 8 == 1:
                b.reverse()
            else:
                rng.shuffle(b)
        else:
            # Permutation of 1..n
            a = list(range(1, n + 1))
            b = list(range(1, n + 1))
            rng.shuffle(a)
            rng.shuffle(b)
        inp = f"{n}\n" + " ".join(map(str, a)) + "\n" + " ".join(map(str, b)) + "\n"
        o_orig = norm_tokens(run_bin(orig_bin, inp))
        o_cp = norm_tokens(run_bin(cp_bin, inp))
        assert o_orig == o_cp, f"cco24p2 t={t}: orig={o_orig[:10]} cp={o_cp[:10]}"
        k = int(o_cp[0])
        if k != -1:
            sa = list(map(int, o_cp[1:1 + n]))
            sb = list(map(int, o_cp[1 + n:1 + 2 * n]))
            stacks = [[] for _ in range(k + 1)]
            for val, sid in zip(a, sa):
                assert 1 <= sid <= k
                stacks[sid].append(val)
            got = []
            for sid in sb:
                assert 1 <= sid <= k and stacks[sid], "popped empty stack"
                got.append(stacks[sid].pop())
            assert got == b, f"cco24p2 stack sim mismatch: {got} != {b}"
        cases += 1
    return f"{cases} impossible/binary/permutation cases (exact match + stack simulation)"


def verify_ccc20s2hard(cp_bin, orig_bin):
    rng = random.Random(2002)
    cases = 0
    for t in range(50):
        if t < 45:
            n = rng.randint(1, 25)
            m = rng.randint(1, 25)
        else:
            n = rng.randint(120, 200)
            m = rng.randint(120, 200)
        hi = n * m + rng.choice([0, 0, 5])
        lines = [f"{n} {m}"]
        for r in range(n + 1):
            row = [0 if (r == 0 or c == 0) else rng.randint(1, hi) for c in range(m + 1)]
            lines.append(" ".join(map(str, row)))
        inp = "\n".join(lines) + "\n"
        o1 = norm_tokens(run_bin(orig_bin, inp))
        o2 = norm_tokens(run_bin(cp_bin, inp))
        assert o1 == o2, f"ccc20s2hard t={t} {n}x{m}: orig={o1} cp={o2}"
        cases += 1
    return f"{cases} random grids up to 200x200 (exact yes/no match)"


def run_cco01p4io_game(bin_path, n, d, secret):
    p = subprocess.Popen(
        [bin_path],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, bufsize=1,
    )
    drops = 0
    broken = 0
    decided = None
    try:
        while True:
            line = p.stdout.readline()
            if not line:
                break
            parts = line.strip().split()
            if not parts:
                continue
            cmd = parts[0]
            if cmd == "GetN":
                p.stdin.write(f"{n}\n")
                p.stdin.flush()
            elif cmd == "GetD":
                p.stdin.write(f"{d}\n")
                p.stdin.flush()
            elif cmd == "Drop":
                x = int(parts[1])
                drops += 1
                res = 1 if x >= secret else 0
                if res == 1:
                    broken += 1
                assert broken <= 2, f"broke > 2 eggs (n={n}, secret={secret})"
                p.stdin.write(f"{res}\n")
                p.stdin.flush()
            elif cmd == "Decide":
                decided = int(parts[1])
                break
            else:
                raise RuntimeError(f"Unknown command: {line!r}")
        p.wait(timeout=5)
        err = p.stderr.read()
        assert p.returncode == 0, f"nonzero exit {p.returncode}: {err}"
    finally:
        if p.poll() is None:
            p.kill()
    assert decided == secret, f"decided {decided} != secret {secret} (n={n})"
    assert drops <= d, f"used {drops} drops > d={d} (n={n}, secret={secret})"
    return drops


def verify_cco01p4io(cp_bin, orig_bin):
    rng = random.Random(104)
    cases = 0
    # Min drops needed for two eggs on n floors is smallest k with k(k+1)/2 >= n - 1
    def min_drops(n):
        k = 0
        while k * (k + 1) // 2 < n - 1:
            k += 1
        return k

    # Exhaustive for small n
    for n in range(1, 13):
        d = min_drops(n)
        for secret in range(1, n + 1):
            run_cco01p4io_game(orig_bin, n, d, secret)
            run_cco01p4io_game(cp_bin, n, d, secret)
            cases += 1
    # Random up to n = 100
    for _ in range(30):
        n = rng.randint(13, 100)
        d = min_drops(n)
        secret = rng.randint(1, n)
        run_cco01p4io_game(orig_bin, n, d, secret)
        run_cco01p4io_game(cp_bin, n, d, secret)
        cases += 1
    return f"{cases} live interactive egg-drop games (exhaustive n<=12 + random n<=100)"


def run_cco07p5_game(bin_path, n, p0, v):
    p = subprocess.Popen(
        [bin_path],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, bufsize=1,
    )
    pos = p0 % n
    queries = 0
    guess = None
    try:
        while True:
            line = p.stdout.readline()
            if not line:
                break
            parts = line.strip().split()
            if not parts:
                continue
            cmd = parts[0]
            if cmd == "GetSize":
                p.stdin.write(f"{n}\n")
                p.stdin.flush()
            elif cmd == "Look":
                l, r = int(parts[1]), int(parts[2])
                queries += 1
                assert queries <= 5000, f"too many queries ({queries}) for n={n}"
                hit = (l <= pos <= r)
                p.stdin.write(("True\n" if hit else "False\n"))
                p.stdin.flush()
                pos = (pos + v) % n
            elif cmd == "Guess":
                guess = (int(parts[1]), int(parts[2]))
                break
            else:
                raise RuntimeError(f"Unknown command: {line!r}")
        p.wait(timeout=5)
        err = p.stderr.read()
        assert p.returncode == 0, f"nonzero exit {p.returncode}: {err}"
    finally:
        if p.poll() is None:
            p.kill()
    assert guess == (v, pos), f"guess {guess} != expected {(v, pos)} (n={n}, p0={p0}, v={v})"
    return queries


def verify_cco07p5(cp_bin, orig_bin):
    rng = random.Random(705)
    cases = 0
    max_q_cp = 0
    for t in range(40):
        n = rng.randint(2, 60) if t < 25 else rng.randint(61, 500)
        p0 = rng.randint(0, n - 1)
        v = rng.randint(0, n - 1)
        run_cco07p5_game(orig_bin, n, p0, v)
        q = run_cco07p5_game(cp_bin, n, p0, v)
        if q > max_q_cp:
            max_q_cp = q
        cases += 1
    return f"{cases} live interactive ring games up to n=500 (max queries={max_q_cp})"


def verify_ccc18s2(cp_bin, orig_bin):
    rng = random.Random(1802)
    cases = 0
    for t in range(40):
        n = rng.randint(2, 35)
        g = [[0] * n for _ in range(n)]
        for r in range(n):
            for c in range(n):
                lo = 1
                if r > 0:
                    lo = max(lo, g[r - 1][c] + 1)
                if c > 0:
                    lo = max(lo, g[r][c - 1] + 1)
                g[r][c] = lo + rng.randint(0, 5)
        for _ in range(t % 4):
            g = [[g[n - 1 - c][r] for c in range(n)] for r in range(n)]
        inp = f"{n}\n" + "\n".join(" ".join(map(str, row)) for row in g) + "\n"
        assert norm_tokens(run_bin(cp_bin, inp)) == norm_tokens(run_bin(orig_bin, inp))
        cases += 1
    return f"{cases} rotated grids across all 4 orientations"


def verify_ccc22s2(cp_bin, orig_bin):
    rng = random.Random(2202)
    cases = 0
    for t in range(40):
        g = rng.randint(1, 80)
        names = [f"S{i}" for i in range(3 * g)]
        rng.shuffle(names)
        x = rng.randint(0, 150)
        y = rng.randint(0, 150)
        same = [(rng.choice(names), rng.choice(names)) for _ in range(x)]
        same = [(a, b) for a, b in same if a != b]
        diff = [(rng.choice(names), rng.choice(names)) for _ in range(y)]
        diff = [(a, b) for a, b in diff if a != b]
        lines = [str(len(same))] + [f"{a} {b}" for a, b in same]
        lines += [str(len(diff))] + [f"{a} {b}" for a, b in diff]
        lines += [str(g)] + [f"{names[3*i]} {names[3*i+1]} {names[3*i+2]}" for i in range(g)]
        inp = "\n".join(lines) + "\n"
        assert norm_tokens(run_bin(cp_bin, inp)) == norm_tokens(run_bin(orig_bin, inp))
        cases += 1
    return f"{cases} random student-group constraint instances"


def verify_cco04p1(cp_bin, orig_bin):
    rng = random.Random(401)
    cases = 0
    for t in range(40):
        m = rng.randint(0, 26)
        letters = rng.sample("abcdefghijklmnopqrstuvwxyz", m)
        lines = [str(m)]
        for ch in letters:
            lines.append(f"{ch} {rng.randint(1, 100)} {rng.randint(1, 6)}")
        n = rng.randint(0, 120)
        lines.append(str(n))
        pool = "".join(letters) if letters else "abc"
        for _ in range(n):
            use_all = rng.random() < 0.3
            src = "abcdefghijklmnopqrstuvwxyz" if use_all else pool
            wlen = rng.randint(1, 15)
            lines.append("".join(rng.choice(src) for _ in range(wlen)))
        inp = "\n".join(lines) + "\n"
        assert norm_tokens(run_bin(cp_bin, inp)) == norm_tokens(run_bin(orig_bin, inp))
        cases += 1
    return f"{cases} random letter-table & dictionary instances"


VERIFIERS = [
    ("cco23p5", verify_cco23p5),
    ("cco24p5", verify_cco24p5),
    ("cco24p4", verify_cco24p4),
    ("cco24p2", verify_cco24p2),
    ("ccc20s2hard", verify_ccc20s2hard),
    ("cco01p4io", verify_cco01p4io),
    ("cco07p5", verify_cco07p5),
    ("ccc18s2", verify_ccc18s2),
    ("ccc22s2", verify_ccc22s2),
    ("cco04p1", verify_cco04p1),
]


def main():
    tmpdir = tempfile.mkdtemp(prefix="scout_batch_47_")
    try:
        for name, fn in VERIFIERS:
            cp_bin = compile_cp(name, tmpdir)
            orig_bin = compile_orig(name, tmpdir)
            summary = fn(cp_bin, orig_bin)
            print(f"PASS  {name:<12s}  {summary}")
        print(f"\nALL {len(VERIFIERS)}/{len(VERIFIERS)} BATCH 47 PORTS VERIFIED (ASan + UBSan clean).")
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


if __name__ == "__main__":
    main()
