#!/usr/bin/env python3
"""Verify the Scout Batch 25 today-form ports.

Every port in today/ is compiled with the prototype compiler (proto/cp.py
build) and run under ASan + UBSan against the accepted C++ submission from
corpus/submissions/ on random small and medium inputs. Problems with many
valid answers (ioi16p1, ioi18p4, ioi20p3, noi20p4, ioi22p2) are judged by independent checkers
written here; grader-style IOI submissions get a small stdin/stdout main.
"""

import os
import random
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CXX = ["g++", "-std=c++17", "-O2"]


def sh(cmd, what):
    r = subprocess.run(cmd, capture_output=True, text=True)
    assert r.returncode == 0, f"{what} failed:\n{r.stdout}{r.stderr[-3000:]}"


def build_cp(name, td):
    dst = os.path.join(td, f"{name}.cp")
    shutil.copy(os.path.join(HERE, "today", f"{name}.cp"), dst)
    sh(["python3", os.path.join(ROOT, "proto", "cp.py"), "build", dst], f"cpc {name}")
    exe = os.path.join(td, f"{name}_cp")
    sh(CXX + ["-fsanitize=address,undefined", "-fno-sanitize-recover=all", "-o", exe, f"{td}/{name}.cpp"], f"g++ {name}")
    return exe


def build_orig(sub_id, td, main=None):
    src = os.path.join(ROOT, "corpus", "submissions", f"{sub_id}.cpp")
    if main is not None:
        wrapped = os.path.join(td, f"{sub_id}_main.cpp")
        with open(wrapped, "w") as f:
            f.write(open(src).read() + "\n" + main)
        src = wrapped
    exe = os.path.join(td, f"{sub_id}_orig")
    sh(CXX + ["-o", exe, src], f"g++ original {sub_id}")
    return exe


def run(exe, text):
    # noi20p4's original keeps n + 1 multi-megabit bitsets on the stack
    # (ASan refuses an unlimited stack, so only the originals get one)
    cmd = [exe] if exe.endswith("_cp") else ["bash", "-c", f"ulimit -s unlimited; exec {exe}"]
    r = subprocess.run(cmd, input=text, capture_output=True, text=True, timeout=120)
    assert r.returncode == 0, f"{exe} crashed on:\n{text[:2000]}\n{r.stderr[-3000:]}"
    return r.stdout.split("\n")


def tokens(lines):
    return [ln.split() for ln in lines if ln.strip()]


def same(name, cp, orig, gen, cases):
    for i in range(cases):
        text = gen(i)
        a, b = tokens(run(cp, text)), tokens(run(orig, text))
        assert a == b, f"{name} differs on:\n{text[:2000]}\ncp:   {a[:10]}\norig: {b[:10]}"
    print(f"PASS {name} ({cases} cases, identical to the original C++, ASan/UBSan clean)")


# The submissions `#define long int64_t`, so the mains below use int64_t and
# iostream rather than `long long` and printf formats.
MAIN_BISCUITS = r"""
int main() {
    int q; cin >> q;
    while (q--) {
        int k; int64_t x; cin >> k >> x;
        vector<ll> a(k);
        for (auto& v : a) cin >> v;
        cout << count_tastiness(x, a) << "\n";
    }
}
"""

MAIN_TICKETS = r"""
void allocate_tickets(std::vector<std::vector<int>> s) {
    for (auto& row : s) { for (int v : row) cout << v << " "; cout << "\n"; }
}
int main() {
    int n, m, k; cin >> n >> m >> k;
    vector<vector<int>> x(n, vector<int>(m));
    for (auto& r : x) for (auto& v : r) cin >> v;
    cout << find_maximum(k, x) << "\n";
}
"""

MAIN_MOLECULES = r"""
int main() {
    int n, l, u; cin >> n >> l >> u;
    vector<int> w(n);
    for (auto& v : w) cin >> v;
    vector<int> got = find_subset(l, u, w);
    cout << got.size() << "\n";
    for (int v : got) cout << v << " ";
    cout << "\n";
}
"""

MAIN_DOLL = r"""
void answer(vector<int> C, vector<int> X, vector<int> Y) {
    for (int v : C) cout << v << " ";
    cout << "\n" << X.size();
    for (int v : X) cout << " " << v;
    cout << "\n" << Y.size();
    for (int v : Y) cout << " " << v;
    cout << "\n";
}
int main() {
    int m, n; cin >> m >> n;
    vector<int> a(n);
    for (auto& v : a) cin >> v;
    create_circuit(m, a);
}
"""

MAIN_PRISONER = r"""
int main() {
    int n; cin >> n;
    auto s = devise_strategy(n);
    cout << s.size() << "\n";
    for (auto& row : s) { for (int v : row) cout << v << " "; cout << "\n"; }
}
"""


def main():
    rng = random.Random(25)
    with tempfile.TemporaryDirectory() as td:
        # 1. noip19p1: k-th Gray code word, k up to 2^64 - 1
        def gen(i):
            n = rng.randint(1, 6) if i < 100 else rng.choice([31, 32, 33, 63, 64, rng.randint(1, 64)])
            k = rng.choice([0, (1 << n) - 1, rng.randrange(1 << n), rng.randrange(1 << n)])
            return f"{n} {k}\n"
        same("noip19p1", build_cp("noip19p1", td), build_orig("5127723", td), gen, 300)

        # 2. ioi09p3: three-key ranking
        def gen(i):
            n, t = rng.randint(1, 6 if i < 150 else 40), rng.randint(1, 4 if i < 150 else 30)
            p = rng.choice([0.2, 0.5, 0.9])
            rows = [" ".join(str(int(rng.random() < p)) for _ in range(t)) for _ in range(n)]
            return f"{n} {t} {rng.randint(1, n)}\n" + "\n".join(rows) + "\n"
        same("ioi09p3", build_cp("ioi09p3", td), build_orig("5106576", td), gen, 250)

        # 3. ioi16p1: any subset with total in [l, u]; both must agree on existence
        cp, orig = build_cp("ioi16p1", td), build_orig("4686581", td, MAIN_MOLECULES)
        for i in range(300):
            n = rng.randint(1, 7 if i < 220 else 200)
            base, spread = rng.choice([(1, 3), (1, 30), (10**6, 50), (1, 10**9)])
            w = [base + rng.randint(0, spread) for _ in range(n)]
            l = rng.randint(1, max(1, sum(w) + 3))
            u = l + (max(w) - min(w)) + rng.choice([0, 0, 1, 7])
            if u >= 2**31:        # the grader passes l and u as int
                continue
            text = f"{n} {l} {u}\n{' '.join(map(str, w))}\n"
            found = []
            for exe in (cp, orig):
                rows = tokens(run(exe, text))
                idx = list(map(int, rows[1])) if len(rows) > 1 else []
                assert len(idx) == int(rows[0][0]) and len(set(idx)) == len(idx)
                assert not idx or l <= sum(w[j] for j in idx) <= u, f"ioi16p1 bad subset on:\n{text}"
                found.append(bool(idx))
            assert found[0] == found[1], f"ioi16p1 existence differs on:\n{text}"
        print("PASS ioi16p1 (up to 300 cases, subset checked, existence equals the original C++)")

        # 4. ioi18p4: circuit simulated; switch count within n + log2(n)
        cp, orig = build_cp("ioi18p4", td), build_orig("4724581", td, MAIN_DOLL)
        for i in range(150):
            m = rng.randint(1, 5 if i < 100 else 300)
            n = rng.choice([1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, rng.randint(1, 40)]) if i < 100 else rng.randint(1, 3000)
            a = [rng.randint(1, m) for _ in range(n)]
            text = f"{m} {n}\n{' '.join(map(str, a))}\n"
            for label, exe in (("cp", cp), ("orig", orig)):
                rows = tokens(run(exe, text))
                c = list(map(int, rows[0]))
                x, y = (list(map(int, r[1:])) for r in rows[1:3])
                assert len(c) == m + 1 and len(x) == len(y) == int(rows[1][0]) == int(rows[2][0])
                assert len(x) <= n + max(n, 1).bit_length(), f"ioi18p4 {label}: {len(x)} switches for n = {n}"
                state, seen, at, steps = [0] * len(x), [], c[0], 0
                while at != 0:
                    steps += 1
                    assert steps <= 20 * (n + 20), f"ioi18p4 {label} does not return on:\n{text}"
                    if at > 0:
                        seen.append(at)
                        at = c[at]
                    else:
                        sw = -at - 1
                        at = y[sw] if state[sw] else x[sw]
                        state[sw] ^= 1
                assert seen == a and not any(state), f"ioi18p4 {label} wrong on:\n{text}"
        print("PASS ioi18p4 (150 cases, circuit simulated for cp and the original)")

        # 5. ioi20p4: grader function behind a tiny main
        def gen(i):
            q = 5
            out = [str(q)]
            for _ in range(q):
                k = rng.randint(1, 6 if i < 100 else 60)
                budget = rng.choice([40, 10**6, 10**18])
                a = []
                for b in range(k):
                    v = rng.randint(0, max(0, budget >> b)) if rng.random() < 0.7 else 0
                    v = min(v, rng.choice([3, 10**18]))
                    budget -= v << b
                    a.append(v)
                x = rng.choice([1, 2, 3, rng.randint(1, 50), rng.randint(1, 10**18)])
                out.append(f"{k} {x}\n" + " ".join(map(str, a)))
            return "\n".join(out) + "\n"
        same("ioi20p4", build_cp("ioi20p4", td), build_orig("4733047", td, MAIN_BISCUITS), gen, 200)

        # 6. ioi20p3: any optimal allocation; checker recomputes the prize
        cp, orig = build_cp("ioi20p3", td), build_orig("4724579", td, MAIN_TICKETS)
        for i in range(200):
            n = 2 * rng.randint(1, 3 if i < 120 else 10)
            m = rng.randint(1, 4 if i < 120 else 12)
            k = rng.randint(1, m)
            top = rng.choice([1, 5, 10**9])
            x = [sorted(rng.randint(0, top) for _ in range(m)) for _ in range(n)]
            text = f"{n} {m} {k}\n" + "\n".join(" ".join(map(str, r)) for r in x) + "\n"
            prizes = []
            for exe, prize_first in [(cp, True), (orig, False)]:
                rows = tokens(run(exe, text))
                prize = int(rows[0][0] if prize_first else rows[-1][0])
                alloc = [list(map(int, r)) for r in (rows[1:] if prize_first else rows[:-1])]
                assert len(alloc) == n and all(len(r) == m for r in alloc)
                total = 0
                for r in range(k):
                    vals = sorted(x[c][t] for c in range(n) for t in range(m) if alloc[c][t] == r)
                    assert len(vals) == n and all(alloc[c].count(r) == 1 for c in range(n)), text
                    total += sum(vals[n // 2:]) - sum(vals[:n // 2])
                assert all(v in range(-1, k) for r in alloc for v in r)
                assert total == prize, f"ioi20p3 prize mismatch on:\n{text}"
                prizes.append(prize)
            assert prizes[0] == prizes[1], f"ioi20p3 not optimal on:\n{text}"
        print("PASS ioi20p3 (200 cases, allocation checked, prize equals the original C++)")

        # 7. btoi19p6: C-th best team
        def gen(i):
            k = rng.randint(1, 4 if i < 150 else 6)
            n = rng.randint(k, k + 4 if i < 150 else 14)
            total = 1
            for j in range(k):
                total = total * (n - j) // (j + 1)
            c = rng.randint(1, min(total, 2000))
            top = rng.choice([1, 3, 10**6])
            rows = [" ".join(str(rng.randint(0, top)) for _ in range(k)) for _ in range(n)]
            return f"{n} {k} {c}\n" + "\n".join(rows) + "\n"
        same("btoi19p6", build_cp("btoi19p6", td), build_orig("4475247", td), gen, 200)

        # 8. noi20p4: any valid plan; both must agree on feasibility
        cp, orig = build_cp("noi20p4", td), build_orig("3890337", td)
        for i in range(250):
            n = rng.randint(1, 7 if i < 180 else 40)
            k = rng.randint(1, 9 if i < 180 else 60)
            m = max(1, n - 2 + rng.choice([0, 0, 0, 1, 2, 5]))
            # n positive amounts summing to m * k
            if m * k < n:
                continue
            cuts = sorted(rng.sample(range(1, m * k), n - 1)) if n > 1 else []
            d = [b - a for a, b in zip([0] + cuts, cuts + [m * k])]
            text = f"1\n{n} {m} {k}\n{' '.join(map(str, d))}\n"
            verdicts = []
            for exe in (cp, orig):
                rows = tokens(run(exe, text))
                if rows == [["-1"]]:
                    verdicts.append(False)
                    continue
                assert len(rows) == m, f"noi20p4 wrong dish count on:\n{text}"
                left = d[:]
                for row in rows:
                    v = list(map(int, row))
                    assert len(v) in (2, 4) and sum(v[1::2]) == k and all(g > 0 for g in v[1::2])
                    assert len(set(v[0::2])) == len(v) // 2
                    for ing, g in zip(v[0::2], v[1::2]):
                        left[ing - 1] -= g
                assert not any(left), f"noi20p4 leftovers on:\n{text}"
                verdicts.append(True)
            assert verdicts[0] == verdicts[1], f"noi20p4 feasibility differs on:\n{text}"
        print("PASS noi20p4 (250 cases, plans checked, feasibility equals the original C++)")

        # 9. ioi22p2: strategy simulated on bag pairs
        cp, orig = build_cp("ioi22p2", td), build_orig("4783608", td, MAIN_PRISONER)
        for n in [2, 3, 4, 5, 9, 10, 11, 22, 23, 67, 68, 69, 207, 5000]:
            for label, exe in (("cp", cp), ("orig", orig)):
                rows = tokens(run(exe, f"{n}\n"))
                s = [list(map(int, r)) for r in rows[1:]]
                assert len(s) == int(rows[0][0]) <= 21, f"ioi22p2 {label}: x = {len(s) - 1} at N = {n}"
                assert all(len(r) == n + 1 and r[0] in (0, 1) for r in s)
                assert all(-2 <= v < len(s) for r in s for v in r[1:])
                if n <= 250:
                    pairs = [(a, b) for a in range(1, n + 1) for b in range(1, n + 1) if a != b]
                else:
                    pairs = [(a, a + e) for a in range(1, n + 1) for e in (-1, 1) if 1 <= a + e <= n]
                    pairs += [tuple(rng.sample(range(1, n + 1), 2)) for _ in range(40000)]
                for a, b in pairs:
                    state = 0
                    for _ in range(len(s) + 1):
                        act = s[state][a if s[state][0] == 0 else b]
                        if act < 0:
                            break
                        state = act
                    assert act == (-1 if a < b else -2), f"ioi22p2 {label} wrong at N = {n}, bags {a} {b}"
        print("PASS ioi22p2 (14 values of N, strategy simulated for cp and the original, x <= 20)")

        # 10. dmopc21c7p3: XOR step on a wire graph, -1 on a reachable cycle
        def gen(i):
            n = rng.randint(1, 7 if i < 200 else 60)
            m = rng.randint(0, n * 2)
            lit = [int(rng.random() < rng.choice([0.2, 0.7])) for _ in range(n)]
            edges = set()
            for _ in range(m):
                u, v = rng.randint(1, n), rng.randint(1, n)
                if u == v:
                    continue
                if rng.random() < 0.8 and u > v:
                    u, v = v, u
                edges.add((u, v))
            edges = list(edges)
            rng.shuffle(edges)
            return f"{n} {len(edges)}\n{' '.join(map(str, lit))}\n" + "".join(f"{u} {v}\n" for u, v in edges)
        same("dmopc21c7p3", build_cp("dmopc21c7p3", td), build_orig("4453037", td), gen, 300)

    print("ALL 10 PASS")


if __name__ == "__main__":
    main()
