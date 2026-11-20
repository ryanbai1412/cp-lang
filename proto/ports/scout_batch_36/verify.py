#!/usr/bin/env python3
"""Verify the Scout Batch 36 (CCC/CCO) today-form ports against original C++.

Nine ports in today/ are built with proto/cp.py build on a temporary copy,
compiled with g++ -std=c++17 -O2 -fsanitize=address,undefined, and checked
against the accepted corpus C++ submission on small and medium random inputs.
cco19p4 uses floating-point arithmetic (F64 / sqrt), which proto/cpc lacks,
so today/cco19p4.cp is verified via the reference interpreter (proto/cp.py
lib-run) against 3617807.cpp within floating-point tolerance.
"""

import math
import os
import random
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CXX = ["g++", "-std=c++17", "-O2"]


def sh(cmd, what):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise AssertionError(f"{what} failed:\n{r.stdout}\n{r.stderr[-3000:]}")


def build_cp(name, td):
    dst = os.path.join(td, f"{name}.cp")
    shutil.copy(os.path.join(HERE, "today", f"{name}.cp"), dst)
    sh(["python3", os.path.join(ROOT, "proto", "cp.py"), "build", dst], f"cpc {name}")
    exe = os.path.join(td, f"{name}_cp")
    sh(
        CXX + ["-fsanitize=address,undefined", "-fno-sanitize-recover=all", "-o", exe, f"{td}/{name}.cpp"],
        f"g++ {name}",
    )
    return exe


def build_orig(sub_id, td):
    src = os.path.join(ROOT, "corpus", "submissions", f"{sub_id}.cpp")
    exe = os.path.join(td, f"{sub_id}_orig")
    sh(CXX + ["-o", exe, src], f"g++ original {sub_id}")
    return exe


def run_bin(exe, text):
    cmd = [exe] if exe.endswith("_cp") else ["bash", "-c", f"ulimit -s unlimited; exec {exe}"]
    r = subprocess.run(cmd, input=text, capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        raise AssertionError(f"{exe} crashed on:\n{text[:1500]}\n{r.stderr[-2000:]}")
    return r.stdout


def run_cpi(name, text):
    src = os.path.join(HERE, "today", f"{name}.cp")
    r = subprocess.run(
        ["python3", os.path.join(ROOT, "proto", "cp.py"), "lib-run", src],
        input=text,
        capture_output=True,
        text=True,
        timeout=60,
    )
    if r.returncode != 0:
        raise AssertionError(f"lib-run {name} crashed on:\n{text[:1500]}\n{r.stdout}\n{r.stderr[-2000:]}")
    return r.stdout


def lines(out):
    return [ln.strip() for ln in out.splitlines() if ln.strip()]


def verify_ccc02s4hard(td):
    cp = build_cp("ccc02s4hard", td)
    orig = build_orig("4157744", td)
    rng = random.Random(3601)
    cases = 120
    for tc in range(cases):
        if tc == 0:
            m, n = 1, 1
        elif tc == 1:
            m, n = 1, 10
        elif tc == 2:
            m, n = 10, 5
        else:
            n = rng.randint(1, 35)
            m = rng.randint(1, min(n + 2, 10))
        people = [(f"p{i}", rng.randint(1, 40)) for i in range(n)]
        if tc == 3:
            people = [(f"p{i}", i + 1) for i in range(n)]
        elif tc == 4:
            people = [(f"p{i}", n - i) for i in range(n)]
        inp = f"{m}\n{n}\n" + "\n".join(f"{nm}\n{tm}" for nm, tm in people) + "\n"
        out_cp = lines(run_bin(cp, inp))
        out_or = lines(run_bin(orig, inp))
        assert out_cp[0] == out_or[0], f"ccc02s4hard header mismatch:\n{out_cp[0]} vs {out_or[0]}"
        want = int(out_or[0].split()[-1])
        pos = 0
        tot = 0
        for row in out_cp[1:]:
            toks = row.split()
            assert 1 <= len(toks) <= m, f"bad group size {len(toks)} > {m}"
            mx = 0
            for tok in toks:
                assert pos < n and tok == people[pos][0], f"name mismatch at {pos}: {tok}"
                mx = max(mx, people[pos][1])
                pos += 1
            tot += mx
        assert pos == n and tot == want, f"ccc02s4hard invalid schedule: pos={pos}/{n}, tot={tot}/{want}"
    print(f"PASS ccc02s4hard ({cases} cases, optimal total + valid grouping checked, ASan/UBSan clean)")


def verify_ccc19s5hard(td):
    cp = build_cp("ccc19s5hard", td)
    orig = build_orig("5119931", td)
    rng = random.Random(3602)
    cases = 100
    for tc in range(cases):
        if tc == 0:
            n, k = 1, 1
        elif tc == 1:
            n, k = 10, 1
        elif tc == 2:
            n, k = 10, 10
        else:
            n = rng.randint(1, 55)
            k = rng.randint(1, n)
        rows = [f"{n} {k}"]
        for i in range(n):
            rows.append(" ".join(str(rng.randint(0, 10_000_000)) for _ in range(i + 1)))
        inp = "\n".join(rows) + "\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"ccc19s5hard differs on tc={tc} (n={n}, k={k}): cp={a} orig={b}"
    print(f"PASS ccc19s5hard ({cases} cases, identical to original C++, ASan/UBSan clean)")


def verify_cco18p6(td):
    cp = build_cp("cco18p6", td)
    orig = build_orig("4575245", td)
    rng = random.Random(3603)
    cases = 100
    for tc in range(cases):
        n = 1 if tc == 0 else (2 if tc == 1 else rng.randint(2, 45))
        a = list(range(1, n + 1))
        b = list(range(1, n + 1))
        if tc == 2:
            b.reverse()
        elif tc > 2:
            rng.shuffle(a)
            rng.shuffle(b)
        inp = f"{n}\n" + " ".join(map(str, a)) + "\n" + " ".join(map(str, b)) + "\n"
        out_cp = lines(run_bin(cp, inp))
        out_or = lines(run_bin(orig, inp))
        assert out_cp == out_or, f"cco18p6 differs on tc={tc}:\ncp={out_cp[:10]}\norig={out_or[:10]}"
        cnt = int(out_cp[0])
        assert len(out_cp) == cnt + 1 and cnt <= 300_000
        cur = list(a)
        for row in out_cp[1:]:
            x, y = map(int, row.split())
            assert 1 <= x <= n and 1 <= y <= n and x != y
            cur[x - 1], cur[y - 1] = cur[y - 1], cur[x - 1]
        assert cur == b, f"cco18p6 swaps did not reach target: {cur} vs {b}"
    print(f"PASS cco18p6 ({cases} cases, swap simulation + exact match verified, ASan/UBSan clean)")


def verify_cco19p4(td):
    orig = build_orig("3617807", td)
    rng = random.Random(3604)
    cases = 45
    for tc in range(cases):
        k = [2, 3, 4][tc % 3]
        n = rng.randint(1, 30)
        max_r = rng.randint(1, min(n, 6))
        arr = [rng.randint(1, max_r) for _ in range(n)]
        inp = f"{k}\n{n}\n" + "\n".join(map(str, arr)) + "\n"
        out_cp = float(lines(run_cpi("cco19p4", inp))[0])
        out_or = float(lines(run_bin(orig, inp))[0])
        assert math.isclose(out_cp, out_or, rel_tol=1e-5, abs_tol=1e-5), (
            f"cco19p4 differs on k={k} arr={arr}: cp={out_cp} orig={out_or}"
        )
    print(f"PASS cco19p4 ({cases} cases via proto/cpi lib-run, matches original within 1e-5)")


def verify_cco20p5(td):
    cp = build_cp("cco20p5", td)
    orig = build_orig("4562195", td)
    rng = random.Random(3605)
    cases = 100
    for tc in range(cases):
        q = rng.randint(1, 100)
        max_c = rng.choice([10, 40, 200])
        active = []
        evs = []
        for _ in range(q):
            if len(active) <= 1 or rng.random() < 0.65:
                l = rng.randint(1, max_c - 1)
                r = rng.randint(l + 1, max_c)
                if active and rng.random() < 0.25:
                    l, r = rng.choice(active)  # duplicate intervals
                active.append((l, r))
                evs.append(f"A {l} {r}")
            else:
                idx = rng.randrange(len(active))
                l, r = active.pop(idx)
                evs.append(f"R {l} {r}")
        inp = f"{q}\n" + "\n".join(evs) + "\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"cco20p5 differs on tc={tc}:\ncp={a[:15]}\norig={b[:15]}"
    print(f"PASS cco20p5 ({cases} cases, identical to original C++, ASan/UBSan clean)")


def verify_cco22p2(td):
    cp = build_cp("cco22p2", td)
    orig = build_orig("4801882", td)
    rng = random.Random(3606)
    cases = 120
    for tc in range(cases):
        n = 2 if tc == 0 else rng.randint(2, 35)
        bus = [rng.randint(0, 40) for _ in range(n)]
        ppl = [rng.randint(0, 50) for _ in range(n - 1)]
        umbr = [rng.randint(0, 35) for _ in range(n - 1)]
        inp = (
            f"{n}\n"
            + " ".join(map(str, bus)) + "\n"
            + " ".join(map(str, ppl)) + "\n"
            + " ".join(map(str, umbr)) + "\n"
        )
        out_cp = lines(run_bin(cp, inp))
        out_or = lines(run_bin(orig, inp))
        assert out_cp[0] == out_or[0], f"cco22p2 YES/NO mismatch on tc={tc}: {out_cp[0]} vs {out_or[0]}"
        if out_cp[0] == "YES":
            assert out_cp[1] == out_or[1], f"cco22p2 umbrella total mismatch: {out_cp[1]} vs {out_or[1]}"
            want_u = int(out_or[1])
            rows = [list(map(int, r.split())) for r in out_cp[2:]]
            assert len(rows) == n - 1
            bus_used = [0] * n
            tot_u = 0
            for i, (x, y, z) in enumerate(rows):
                assert x >= 0 and y >= 0 and z >= 0
                assert x + y + z == ppl[i], f"market {i} sum mismatch"
                assert y <= umbr[i], f"market {i} umbrellas exceeded"
                bus_used[i] += x
                bus_used[i + 1] += z
                tot_u += y
            assert tot_u == want_u
            assert all(bus_used[i] <= bus[i] for i in range(n)), f"bus cap exceeded: {bus_used} > {bus}"
    print(f"PASS cco22p2 ({cases} cases, optimal total + valid allocation checked, ASan/UBSan clean)")


def verify_cco22p4(td):
    cp = build_cp("cco22p4", td)
    orig = build_orig("4801884", td)
    rng = random.Random(3607)
    cases = 120
    for tc in range(cases):
        n = 1 if tc == 0 else rng.randint(1, 30)
        kids = [[0, 0] for _ in range(n + 1)]
        slots = [(1, 0), (1, 1)]
        for v in range(2, n + 1):
            idx = rng.randrange(len(slots))
            p, side = slots.pop(idx)
            kids[p][side] = v
            slots.append((v, 0))
            slots.append((v, 1))
        m = rng.randint(0, n + 2)
        balls = [rng.randint(1, n) for _ in range(m)]
        buf = [f"{n} {m}", " ".join(map(str, balls))]
        for u in range(1, n + 1):
            buf.append(f"{kids[u][0]} {kids[u][1]}")
        inp = "\n".join(buf) + "\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"cco22p4 differs on tc={tc}:\n{inp}\ncp={a} orig={b}"
    print(f"PASS cco22p4 ({cases} cases, identical to original C++, ASan/UBSan clean)")


def verify_cco22p5(td):
    cp = build_cp("cco22p5", td)
    orig = build_orig("4801885", td)
    rng = random.Random(3608)
    cases = 120
    for tc in range(cases):
        n = rng.randint(1, 20)
        ma = 0 if tc == 0 else rng.randint(0, 30)
        mb = 0 if tc == 1 else rng.randint(0, 30)
        max_pairs = n * (n - 1) // 2
        k = 0 if tc == 2 else rng.randint(0, max_pairs + 3)
        buf = [f"{n} {ma} {mb} {k}"]
        for _ in range(ma):
            buf.append(f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(0, 50)}")
        for _ in range(mb):
            buf.append(f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(0, 50)}")
        inp = "\n".join(buf) + "\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"cco22p5 differs on tc={tc}:\n{inp}\ncp={a} orig={b}"
    print(f"PASS cco22p5 ({cases} cases, identical to original C++, ASan/UBSan clean)")


def verify_cco23p2(td):
    cp = build_cp("cco23p2", td)
    orig = build_orig("5579928", td)
    rng = random.Random(3609)
    fixed = [[5], [3, 3, 3], [5, 1, 5], [10, 2, 3, 1, 8], [1, 2, 3, 4, 5], [5, 4, 3, 2, 1]]
    cases = 120
    for tc in range(cases):
        if tc < len(fixed):
            arr = fixed[tc]
        else:
            n = rng.randint(1, 50)
            hi = rng.choice([5, 25, 1000, 10_000_000])
            arr = [rng.randint(1, hi) for _ in range(n)]
        inp = f"{len(arr)}\n" + " ".join(map(str, arr)) + "\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"cco23p2 differs on arr={arr}: cp={a} orig={b}"
    print(f"PASS cco23p2 ({cases} cases, identical to original C++, ASan/UBSan clean)")


def verify_cco23p4(td):
    cp = build_cp("cco23p4", td)
    orig = build_orig("5579929", td)
    rng = random.Random(3610)
    targets = ["0", "1", "00", "11", "01", "10", "010", "101", "001", "110", "011", "100", "000", "111"]
    cases = 140
    for tc in range(cases):
        t = targets[tc % len(targets)]
        if tc < 14:
            s = "0" * 6 if tc % 2 == 0 else "1" * 6
        elif tc < 28:
            s = "01" * 10
        else:
            n = rng.randint(1, 80)
            p1 = rng.choice([0.15, 0.35, 0.5, 0.7, 0.85])
            s = "".join("1" if rng.random() < p1 else "0" for _ in range(n))
        inp = f"{s}\n{t}\n"
        a = lines(run_bin(cp, inp))
        b = lines(run_bin(orig, inp))
        assert a == b, f"cco23p4 differs on s={s} t={t}: cp={a} orig={b}"
    print(f"PASS cco23p4 ({cases} cases, all 14 targets covered, ASan/UBSan clean)")


ALL = {
    "ccc02s4hard": verify_ccc02s4hard,
    "ccc19s5hard": verify_ccc19s5hard,
    "cco18p6": verify_cco18p6,
    "cco19p4": verify_cco19p4,
    "cco20p5": verify_cco20p5,
    "cco22p2": verify_cco22p2,
    "cco22p4": verify_cco22p4,
    "cco22p5": verify_cco22p5,
    "cco23p2": verify_cco23p2,
    "cco23p4": verify_cco23p4,
}


def main():
    wanted = sys.argv[1:] if len(sys.argv) > 1 else list(ALL)
    with tempfile.TemporaryDirectory(prefix="scout36_") as td:
        for name in wanted:
            ALL[name](td)


if __name__ == "__main__":
    main()
