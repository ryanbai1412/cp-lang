#!/usr/bin/env python3
"""Verify all 9 buildable Stage-1 Scout Batch 19 today-form ports under ASan+UBSan
against their original C++ submissions and independent interactors/checkers."""

import math
import os
import random
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))


def build_cp(name, td):
    src = os.path.join(HERE, "today", f"{name}.cp")
    dst = os.path.join(td, f"{name}.cp")
    shutil.copy(src, dst)
    r = subprocess.run(
        ["python3", os.path.join(ROOT, "proto", "cp.py"), "build", dst],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, f"cpc failed on {name}:\n{r.stdout}{r.stderr}"
    exe = os.path.join(td, f"{name}_cp")
    r2 = subprocess.run(
        ["g++", "-std=c++17", "-O2", "-fsanitize=address,undefined", "-o", exe, f"{td}/{name}.cpp"],
        capture_output=True,
        text=True,
    )
    assert r2.returncode == 0, f"g++ failed on {name}:\n{r2.stderr}"
    return exe


def build_cpp(sub_id, td):
    exe = os.path.join(td, f"{sub_id}_orig")
    src = os.path.join(ROOT, "corpus", "submissions", f"{sub_id}.cpp")
    r = subprocess.run(["g++", "-std=c++17", "-O2", "-o", exe, src], capture_output=True, text=True)
    assert r.returncode == 0, f"g++ orig failed on {sub_id}:\n{r.stderr}"
    return exe


def is_prime(x):
    if x < 2:
        return False
    if x % 2 == 0:
        return x == 2
    d = 3
    while d * d <= x:
        if x % d == 0:
            return False
        d += 2
    return True


def main():
    rng = random.Random(19)
    with tempfile.TemporaryDirectory() as td:
        # 1. egoi23p3 (interactive ?/!, 2-query limit)
        cp_exe = build_cp("egoi23p3", td)
        orig_exe = build_cpp("5678051", td)
        for _ in range(30):
            h = rng.randint(1, 15)
            w = rng.randint(1, 15)
            if h == 1 and w == 1:
                w = 2
            cells = [(r, c) for r in range(h) for c in range(w) if (r, c) != (0, 0)]
            box = rng.choice(cells)
            for label, exe in [("cp", cp_exe), ("orig", orig_exe)]:
                p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
                p.stdin.write(f"{h} {w}\n")
                p.stdin.flush()
                qs = 0
                ok = False
                for line in p.stdout:
                    parts = line.strip().split()
                    if not parts:
                        continue
                    if parts[0] == "?":
                        qs += 1
                        assert qs <= 2, f"egoi23p3 {label} exceeded 2 queries"
                        moves = parts[1]
                        assert len(moves) <= 20000
                        r, c = 0, 0
                        for ch in moves:
                            nr, nc = r, c
                            if ch == "<":
                                nc -= 1
                            elif ch == ">":
                                nc += 1
                            elif ch == "^":
                                nr -= 1
                            elif ch == "v":
                                nr += 1
                            if 0 <= nr < h and 0 <= nc < w and (nr, nc) != box:
                                r, c = nr, nc
                        p.stdin.write(f"{r} {c}\n")
                        p.stdin.flush()
                    elif parts[0] == "!":
                        assert (int(parts[1]), int(parts[2])) == box
                        ok = True
                        break
                p.wait(timeout=5)
                assert ok and p.returncode == 0
        print("PASS egoi23p3 (30 interactive cases vs orig C++ + ASan/UBSan)")

        # 2. ioi07p1 (interactive examine/solution, 300-query limit)
        cp_exe = build_cp("ioi07p1", td)
        orig_exe = build_cpp("5111683", td)
        for _ in range(30):
            m = rng.choice([1, 2, 3, 5, 7, 11, 25, 101])
            n = rng.randint(5 * m, max(5 * m + 10, 20 * m))
            bx = rng.randint(1, n - 5 * m + 1)
            by = rng.randint(1, n - 5 * m + 1)
            reds = [(i, j) for i in range(5) for j in range(5) if (i + j) % 2 == 0]
            ri, rj = rng.choice(reds)
            x0 = bx + ri * m + rng.randint(0, m - 1)
            y0 = by + rj * m + rng.randint(0, m - 1)
            cx, cy = bx + 2 * m + m // 2, by + 2 * m + m // 2
            for label, exe in [("cp", cp_exe), ("orig", orig_exe)]:
                p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
                p.stdin.write(f"{n} {x0} {y0}\n")
                p.stdin.flush()
                qs = 0
                ok = False
                for line in p.stdout:
                    parts = line.strip().split()
                    if not parts:
                        continue
                    if parts[0] == "examine":
                        qs += 1
                        assert qs <= 300, f"ioi07p1 {label} exceeded 300 queries"
                        qx, qy = int(parts[1]), int(parts[2])
                        assert 1 <= qx <= n and 1 <= qy <= n
                        dx, dy = qx - bx, qy - by
                        red = 0 <= dx < 5 * m and 0 <= dy < 5 * m and ((dx // m) + (dy // m)) % 2 == 0
                        p.stdin.write("true\n" if red else "false\n")
                        p.stdin.flush()
                    elif parts[0] == "solution":
                        assert (int(parts[1]), int(parts[2])) == (cx, cy)
                        ok = True
                        break
                p.wait(timeout=5)
                assert ok and p.returncode == 0
        print("PASS ioi07p1 (30 interactive cases vs orig C++ + ASan/UBSan)")

        # 3. ioi16p5 (two-phase interactive, separate write/read limits)
        cp_exe = build_cp("ioi16p5", td)
        for n in [2, 4, 8, 16, 32, 64]:
            for _ in range(5):
                perm = list(range(n))
                rng.shuffle(perm)
                w_lim = n * int(math.log2(n)) // 2
                r_lim = n * int(math.log2(n))
                p = subprocess.Popen([cp_exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
                p.stdin.write(f"{n} {w_lim} {r_lim}\n")
                p.stdin.flush()
                added = set()
                compiled = False
                wc = rc = 0
                ok = False
                for line in p.stdout:
                    parts = line.strip().split()
                    if not parts:
                        continue
                    if parts[0] == "A":
                        assert not compiled
                        wc += 1
                        assert wc <= w_lim
                        added.add(parts[1])
                        p.stdin.write("1\n")
                        p.stdin.flush()
                    elif parts[0] == "C":
                        assert not compiled
                        compiled = True
                        permuted = {"".join(s[perm[i]] for i in range(n)) for s in added}
                        p.stdin.write("1\n")
                        p.stdin.flush()
                    elif parts[0] == "Q":
                        assert compiled
                        rc += 1
                        assert rc <= r_lim
                        p.stdin.write("1\n" if parts[1] in permuted else "0\n")
                        p.stdin.flush()
                    elif parts[0] == "!":
                        assert list(map(int, parts[1:])) == perm
                        ok = True
                        break
                p.wait(timeout=5)
                assert ok and p.returncode == 0
        print("PASS ioi16p5 (30 two-phase interactive cases + ASan/UBSan)")

        # 4. ioi14p3 (reversed callback adaptive game)
        cp_exe = build_cp("ioi14p3", td)
        for n in [2, 3, 5, 10, 20]:
            for _ in range(4):
                pairs = [(u, v) for u in range(n) for v in range(u + 1, n)]
                rng.shuffle(pairs)
                p = subprocess.Popen([cp_exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
                p.stdin.write(f"{n}\n")
                p.stdin.flush()
                assert p.stdout.readline().strip() == "READY"
                yes_edges = []
                for idx, (u, v) in enumerate(pairs):
                    p.stdin.write(f"{u} {v}\n")
                    p.stdin.flush()
                    ans = int(p.stdout.readline().strip())
                    assert ans in (0, 1)
                    if ans == 1:
                        yes_edges.append((u, v))
                    if idx < len(pairs) - 1:
                        par = list(range(n))

                        def find(x):
                            while par[x] != x:
                                par[x] = par[par[x]]
                                x = par[x]
                            return x

                        for a, b in yes_edges:
                            ra, rb = find(a), find(b)
                            if ra != rb:
                                par[ra] = rb
                        assert len({find(i) for i in range(n)}) > 1
                p.wait(timeout=5)
                assert p.returncode == 0 and len(yes_edges) == n - 1
        print("PASS ioi14p3 (20 adaptive game cases + ASan/UBSan)")

        # 5. ccc23s3 (constructive char grid + IMPOSSIBLE)
        cp_exe = build_cp("ccc23s3", td)
        orig_exe = build_cpp("5350806", td)
        for n in range(2, 7):
            for m in range(2, 7):
                for r in range(n + 1):
                    for c in range(m + 1):
                        inp = f"{n} {m} {r} {c}\n"
                        o1 = subprocess.check_output([cp_exe], input=inp, text=True).strip().splitlines()
                        o2 = subprocess.check_output([orig_exe], input=inp, text=True).strip().splitlines()
                        if o2 == ["IMPOSSIBLE"]:
                            assert o1 == ["IMPOSSIBLE"]
                        else:
                            assert len(o1) == n and all(len(row) == m and all("a" <= ch <= "z" for ch in row) for row in o1)
                            cr = sum(row == row[::-1] for row in o1)
                            cc = sum(
                                "".join(o1[i][j] for i in range(n)) == "".join(o1[n - 1 - i][j] for i in range(n))
                                for j in range(m)
                            )
                            assert (cr, cc) == (r, c)
        print("PASS ccc23s3 (exhaustive n,m in 2..6 vs orig C++ + checker + ASan/UBSan)")

        # 6. dmopc21c5p4 (multi-test constructive permutation + prime adjacent diffs + -1)
        cp_exe = build_cp("dmopc21c5p4", td)
        orig_exe = build_cpp("4223525", td)
        ns = list(range(1, 101)) + [200, 501, 1000, 1001, 5000, 10000]
        inp = f"{len(ns)}\n" + "\n".join(map(str, ns)) + "\n"
        o1 = subprocess.check_output([cp_exe], input=inp, text=True).strip().splitlines()
        o2 = subprocess.check_output([orig_exe], input=inp, text=True).strip().splitlines()
        assert len(o1) == len(ns) and len(o2) == len(ns)
        for n, l1, l2 in zip(ns, o1, o2):
            if l2 == "-1":
                assert l1 == "-1"
            else:
                perm = list(map(int, l1.split()))
                assert sorted(perm) == list(range(1, n + 1))
                for a, b in zip(perm, perm[1:]):
                    assert is_prime(abs(a - b))
        print(f"PASS dmopc21c5p4 ({len(ns)} cases 1..10000 vs orig C++ + checker + ASan/UBSan)")

        # 7. coci21c6p3 (multi-answer constructive graph + op-sequence simulator checker)
        cp_exe = build_cp("coci21c6p3", td)
        orig_exe = build_cpp("4530006", td)
        for trial in range(80):
            n = rng.randint(2, 18)
            m = rng.randint(0, min(n * (n - 1) // 2, 3 * n))
            eset = set()
            while len(eset) < m:
                u = rng.randint(1, n)
                v = rng.randint(1, n)
                if u != v and (u, v) not in eset and (v, u) not in eset:
                    eset.add((u, v))
            edges = list(eset)
            inp = f"{n} {len(edges)}\n" + "".join(f"{u} {v}\n" for u, v in edges)
            o1 = subprocess.check_output([cp_exe], input=inp, text=True).strip().splitlines()
            o2 = subprocess.check_output([orig_exe], input=inp, text=True).strip().splitlines()
            if o2 == ["-1"]:
                assert o1 == ["-1"], f"coci21c6p3 mismatch: cp={o1} orig=-1"
            else:
                k = int(o1[0])
                assert 0 <= k <= 200000 and len(o1) == k + 1
                last_t = [0] * (n + 1)
                last_d = [0] * (n + 1)
                for idx, line in enumerate(o1[1:], 1):
                    u, d = map(int, line.split())
                    assert 1 <= u <= n and d in (0, 1)
                    last_t[u] = idx
                    last_d[u] = d
                for u, v in edges:
                    assert max(last_t[u], last_t[v]) > 0
                    ok = (last_d[u] == 1) if last_t[u] > last_t[v] else (last_d[v] == 0)
                    assert ok, f"coci21c6p3 edge ({u},{v}) wrong orientation"
        print("PASS coci21c6p3 (80 random directed graphs vs orig C++ + op-sim checker + ASan/UBSan)")

        # 8. ccc04s5 (0 0 sentinel-terminated multi-test + mixed '.', '*', '1'..'9' grid)
        cp_exe = build_cp("ccc04s5", td)
        orig_exe = build_cpp("4806811", td)
        parts = []
        for _ in range(40):
            rows = rng.randint(2, 15)
            cols = rng.randint(2, 15)
            g = [["." for _ in range(cols)] for _ in range(rows)]
            keep = [[False] * cols for _ in range(rows)]
            r = rows - 1
            for c in range(cols):
                nr = rows - 1 if c == cols - 1 else rng.randint(0, rows - 1)
                for k in range(min(r, nr), max(r, nr) + 1):
                    keep[k][c] = True
                r = nr
            for i in range(rows):
                for j in range(cols):
                    if not keep[i][j] and rng.random() < 0.25:
                        g[i][j] = "*"
                    elif rng.random() < 0.35:
                        g[i][j] = str(rng.randint(1, 9))
            parts.append(f"{rows} {cols}\n" + "\n".join("".join(row) for row in g) + "\n")
        inp = "".join(parts) + "0 0\n"
        o1 = subprocess.check_output([cp_exe], input=inp, text=True).strip().splitlines()
        o2 = subprocess.check_output([orig_exe], input=inp, text=True).strip().splitlines()
        assert o1 == o2, f"ccc04s5 diff: cp={o1[:5]} orig={o2[:5]}"
        print("PASS ccc04s5 (40 sentinel-terminated multi-test grids vs orig C++ + ASan/UBSan)")

        # 9. ceoi22p2 (recursive expression-tree string I/O)
        cp_exe = build_cp("ceoi22p2", td)
        orig_exe = build_cpp("4727898", td)

        def gen_expr(leaves):
            if leaves == 1:
                return "?"
            l = rng.randint(1, leaves - 1)
            op = rng.choice(["min", "max"])
            return f"{op}({gen_expr(l)},{gen_expr(leaves - l)})"

        for sz in [1, 2, 3, 5, 10, 50, 200, 1000]:
            for _ in range(5):
                expr = gen_expr(sz) + "\n"
                o1 = subprocess.check_output([cp_exe], input=expr, text=True).strip()
                o2 = subprocess.check_output([orig_exe], input=expr, text=True).strip()
                assert o1 == o2
        print("PASS ceoi22p2 (40 random expression trees up to 1000 leaves vs orig C++ + ASan/UBSan)")


if __name__ == "__main__":
    main()
