#!/usr/bin/env python3
"""Triple differential test of Blind Critic #5 ports via the real Rust `cpl` compiler (c521b2c):
  1. Original accepted C++ submission (compiled with g++ -O2)
  2. Rust `cpl build` emitted C++17 (compiled with g++ -std=c++17 -O2 -fsanitize=address,undefined)
  3. Rust `cpl run` interpreter

Usage: python3 proto/ports/blind_critic_5/check.py [name ...] [--iters N]
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(os.path.dirname(HERE))
ROOT = os.path.dirname(PROTO)
CORPUS = os.path.join(ROOT, "corpus", "submissions")
CPL = os.environ.get("CPL") or os.path.join(ROOT, "target", "debug", "cpl")
if not os.path.exists(CPL):
    subprocess.run(["cargo", "build", "-p", "cpl-cli"], cwd=ROOT, check=True)

ORIGINAL = {
    "bsspc21s4": 4152932,
    "noi22p5": 4832632,
    "noi04p3": 5032868,
    "noi08p4": 5144322,
    "noiwc22p3": 4261177,
    "ioi19p5": 5664246,
}

# Thin C++ grader drivers for the two grader-interface problems.
WORD_H = """#pragma once
const char* guess(int num_testcase, int remaining_guesses, char initial_letter, bool* gold, bool* silver);
void init(int num_scramble, const char* scramble);
"""

DRIVERS = {
    "noiwc22p3": r"""
#include "{orig}"
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int nw;
    if (!(cin >> nw)) return 0;
    string flat;
    flat.reserve(nw * 5);
    for (int i = 0; i < nw; i++) {
        string w; cin >> w;
        flat += w;
    }
    init(nw, flat.c_str());
    int nq; cin >> nq;
    for (int i = 0; i < nq; i++) {
        int rem; string init_s, gs, ss;
        cin >> rem >> init_s >> gs >> ss;
        bool g[5], s[5];
        for (int j = 0; j < 5; j++) { g[j] = gs[j] == '1'; s[j] = ss[j] == '1'; }
        const char* res = guess(1, rem, init_s[0], g, s);
        cout << string(res, 5) << "\n";
    }
    return 0;
}
""",
    "ioi19p5": r"""
#include <bits/stdc++.h>
using namespace std;
static int _next_id = 0;
static vector<int> _ops;
static vector<vector<int>> _args;
int add_not(int N) { _ops.push_back(1); _args.push_back({N}); return _next_id++; }
int add_and(vector<int> Ns) { _ops.push_back(2); _args.push_back(Ns); return _next_id++; }
int add_or(vector<int> Ns) { _ops.push_back(3); _args.push_back(Ns); return _next_id++; }
int add_xor(vector<int> Ns) { _ops.push_back(4); _args.push_back(Ns); return _next_id++; }
#include "{orig}"
int main() {
    ios::sync_with_stdio(false); cin.tie(nullptr);
    int r, c, k;
    if (!(cin >> r >> c >> k)) return 0;
    _next_id = r * c;
    construct_network(r, c, k);
    cout << _ops.size() << "\n";
    for (size_t i = 0; i < _ops.size(); i++) {
        cout << _ops[i] << " " << _args[i].size();
        for (int a : _args[i]) cout << " " << a;
        cout << "\n";
    }
    return 0;
}
""",
}


def gen_bsspc21s4(rng):
    n = rng.randint(2, 64)
    m = rng.randint(2, 64)
    k = rng.randint(1, 10)
    lines = [f"{n} {m} {k}"]
    for _ in range(k):
        x1 = rng.randint(1, n)
        x2 = rng.randint(x1, n)
        y1 = rng.randint(1, m)
        y2 = rng.randint(y1, m)
        lines.append(f"{x1} {y1} {x2} {y2}")
    return "\n".join(lines) + "\n"


def gen_noi22p5(rng):
    tcs = rng.randint(1, 3)
    lines = [str(tcs)]
    for _ in range(tcs):
        n = rng.randint(2, 12)
        # Keep distinct values <= n (the original 4832632.cpp has a domain-mixup bug
        # passing n instead of m to ST::update when m > n, which CP Lang's Val domain catches!)
        m = rng.randint(1, n)
        vals = [rng.randint(1, min(n, 8)) for _ in range(m)]
        lines.append(f"{n} {m}")
        for x in vals:
            l = rng.randint(1, n)
            r = rng.randint(l, n)
            lines.append(f"{l} {r} {x}")
    return "\n".join(lines) + "\n"


def gen_noi04p3(rng):
    n = rng.randint(1, 4)
    m = rng.randint(1, 6)
    sh = "".join(rng.choice("EW") for _ in range(n))
    sv = "".join(rng.choice("NS") for _ in range(m))
    ch = [str(rng.randint(1, 20)) for _ in range(n)]
    cv = [str(rng.randint(1, 20)) for _ in range(m)]
    q = rng.randint(1, 5)
    lines = [f"{n} {m}", sh, sv, " ".join(ch), " ".join(cv), str(q)]
    for _ in range(q):
        x1 = rng.randint(1, n)
        y1 = rng.randint(1, m)
        x2 = rng.randint(1, n)
        y2 = rng.randint(1, m)
        lines.append(f"{x1} {y1} {x2} {y2}")
    return "\n".join(lines) + "\n"


def gen_noi08p4(rng):
    n = rng.randint(3, 10)
    m = rng.randint(1, min(4, n - 1))
    kv = round(rng.uniform(0.15, 0.85), 2)
    # Functional graph where node 1 (0-indexed 0) is on a cycle of length >= 2
    clen = rng.randint(2, n)
    perm = list(range(1, n))
    rng.shuffle(perm)
    cyc = [0] + perm[: clen - 1]
    par = [0] * n
    for i in range(clen):
        par[cyc[i]] = cyc[(i + 1) % clen]
    in_tree = list(cyc)
    for node in perm[clen - 1 :]:
        par[node] = rng.choice(in_tree)
        in_tree.append(node)
    arr = [str(round(rng.uniform(1.0, 50.0), 1)) for _ in range(n)]
    lines = [
        f"{n} {m} {kv}",
        " ".join(str(p + 1) for p in par),
        " ".join(arr),
    ]
    return "\n".join(lines) + "\n"


def eval_wordle(guess, ans):
    gold = [guess[i] == ans[i] for i in range(5)]
    silver = [False] * 5
    for i in range(5):
        if not gold[i]:
            for j in range(5):
                if not gold[j] and guess[i] == ans[j]:
                    silver[i] = True
    return "".join("1" if x else "0" for x in gold), "".join("1" if x else "0" for x in silver)


def gen_noiwc22p3(rng):
    # Generate small realistic 5-letter dictionary + a consistent 2-step Wordle sequence
    alphabet = "aeilnrst"
    words = {"leant", "paper"}
    while len(words) < 18:
        words.add("".join(rng.choice(alphabet) for _ in range(5)))
    words = sorted(words)
    secret = rng.choice(words)
    g1, s1 = eval_wordle("leant", secret)
    lines = [str(len(words))] + words + ["2"]
    lines.append(f"5 {secret[0]} 00000 00000")
    lines.append(f"4 {secret[0]} {g1} {s1}")
    return "\n".join(lines) + "\n"


def gen_ioi19p5(rng):
    r = rng.randint(2, 6)
    c = rng.randint(2, 6)
    k = rng.randint(1, r + c - 2)
    return f"{r} {c} {k}\n"


GENS = {
    "bsspc21s4": gen_bsspc21s4,
    "noi22p5": gen_noi22p5,
    "noi04p3": gen_noi04p3,
    "noi08p4": gen_noi08p4,
    "noiwc22p3": gen_noiwc22p3,
    "ioi19p5": gen_ioi19p5,
}


def eval_vision_circuit(out_text, r, c, k):
    """Evaluate a Vision circuit output on EVERY pair of distinct cells (p1, p2)."""
    lines = [ln.strip() for ln in out_text.strip().splitlines() if ln.strip()]
    num_gates = int(lines[0])
    assert len(lines) == num_gates + 1, f"expected {num_gates} gates, got {len(lines)-1}"
    gates = []
    for ln in lines[1:]:
        toks = [int(t) for t in ln.split()]
        op, cnt, args = toks[0], toks[1], toks[2:]
        assert len(args) == cnt
        gates.append((op, args))
    n_cells = r * c
    for p1 in range(n_cells):
        r1, c1 = divmod(p1, c)
        for p2 in range(p1 + 1, n_cells):
            r2, c2 = divmod(p2, c)
            vals = [0] * (n_cells + num_gates)
            vals[p1] = 1
            vals[p2] = 1
            for gi, (op, args) in enumerate(gates):
                if op == 1:
                    v = 1 - vals[args[0]]
                elif op == 2:
                    v = 1
                    for a in args:
                        v &= vals[a]
                elif op == 3:
                    v = 0
                    for a in args:
                        v |= vals[a]
                else:
                    v = 0
                    for a in args:
                        v ^= vals[a]
                vals[n_cells + gi] = v
            got = vals[-1]
            want = 1 if abs(r1 - r2) + abs(c1 - c2) == k else 0
            if got != want:
                return False, f"pair ({r1},{c1})-({r2},{c2}) dist={abs(r1-r2)+abs(c1-c2)} k={k}: got {got}, want {want}"
    return True, f"all {n_cells*(n_cells-1)//2} pairs exact ({num_gates} gates)"


def check_noi04p3_output(inp, orig_out, port_out):
    """For noi04p3, compare exact feasibility and optimal cost (tie-break strings may differ)."""
    ol = orig_out.strip().splitlines()
    pl = port_out.strip().splitlines()
    if ol[0] != pl[0]:
        return False, f"feasibility mismatch: orig={ol[0]!r}, port={pl[0]!r}"
    if ol[0] == "impossible":
        return True, ""
    if ol[1] != pl[1]:
        return False, f"cost mismatch: orig={ol[1]!r}, port={pl[1]!r}"
    return True, ""


def run_bin(cmd, inp, timeout=15):
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"cmd {cmd} failed (rc={r.returncode}):\n{r.stderr}")
    return r.stdout


def test_problem(name, iters, interp_iters):
    with tempfile.TemporaryDirectory() as tmp:
        # 1. Compile original C++
        orig_src = os.path.join(CORPUS, f"{ORIGINAL[name]}.cpp")
        orig_bin = os.path.join(tmp, f"orig_{name}")
        if name in DRIVERS:
            if name == "noiwc22p3":
                with open(os.path.join(tmp, "word.h"), "w") as f:
                    f.write(WORD_H)
            drv = os.path.join(tmp, f"drv_{name}.cpp")
            with open(drv, "w") as f:
                f.write(DRIVERS[name].replace("{orig}", orig_src))
            src_to_compile = drv
        else:
            src_to_compile = orig_src
        r = subprocess.run(
            ["g++", "-std=c++17", "-O2", "-I", tmp, src_to_compile, "-o", orig_bin],
            capture_output=True,
            text=True,
        )
        assert r.returncode == 0, f"orig g++ failed for {name}:\n{r.stderr}"

        # 2. Build CP Lang port via REAL Rust `cpl build` + ASan/UBSan
        cp_src = os.path.join(HERE, f"{name}.cp")
        emitted_cpp = os.path.join(tmp, f"port_{name}.cpp")
        r = subprocess.run([CPL, "build", cp_src, "-o", emitted_cpp], capture_output=True, text=True)
        assert r.returncode == 0, f"cpl build failed for {name}:\n{r.stdout}\n{r.stderr}"
        cpp_bytes = os.path.getsize(emitted_cpp)
        port_bin = os.path.join(tmp, f"port_{name}")
        r = subprocess.run(
            ["g++", "-std=c++17", "-O2", "-fsanitize=address,undefined", emitted_cpp, "-o", port_bin],
            capture_output=True,
            text=True,
        )
        assert r.returncode == 0, f"port g++ ASan/UBSan failed for {name}:\n{r.stderr}"

        # 3. Differential test across random inputs
        rng = random.Random(20260500 + hash(name) % 10000)
        for it in range(iters):
            inp = GENS[name](rng)
            out_orig = run_bin([orig_bin], inp)
            out_build = run_bin([port_bin], inp)
            if name == "ioi19p5":
                r_v, c_v, k_v = map(int, inp.split())
                ok1, msg1 = eval_vision_circuit(out_orig, r_v, c_v, k_v)
                ok2, msg2 = eval_vision_circuit(out_build, r_v, c_v, k_v)
                assert ok1, f"{name} orig failed on iter {it}: {msg1}"
                assert ok2, f"{name} cpl build failed on iter {it}: {msg2}"
            elif name == "noi04p3":
                ok, msg = check_noi04p3_output(inp, out_orig, out_build)
                assert ok, f"{name} cpl build mismatch on iter {it}: {msg}\nINPUT:\n{inp}"
            elif name == "noi08p4":
                v1, v2 = float(out_orig.strip()), float(out_build.strip())
                assert abs(v1 - v2) <= 0.011, f"{name} cpl build float mismatch iter {it}: orig={out_orig!r} build={out_build!r}"
            else:
                assert out_orig.strip() == out_build.strip(), (
                    f"{name} cpl build mismatch on iter {it}:\nORIG:\n{out_orig}\nBUILD:\n{out_build}\nINPUT:\n{inp}"
                )

            if it < interp_iters:
                out_interp = run_bin([CPL, "run", cp_src], inp, timeout=30)
                assert out_interp.strip() == out_build.strip(), (
                    f"{name} cpl run != cpl build on iter {it}:\nRUN:\n{out_interp}\nBUILD:\n{out_build}\nINPUT:\n{inp}"
                )

        print(
            f"[PASS] {name:10s} | orig vs cpl build (ASan/UBSan): {iters}/{iters} | "
            f"cpl run == cpl build: {interp_iters}/{interp_iters} | emitted C++17: {cpp_bytes} B"
        )


def main():
    args = sys.argv[1:]
    iters = 30
    interp_iters = 5
    names = []
    i = 0
    while i < len(args):
        if args[i] == "--iters":
            iters = int(args[i + 1])
            i += 2
        elif args[i] == "--interp-iters":
            interp_iters = int(args[i + 1])
            i += 2
        else:
            names.append(args[i])
            i += 1
    if not names:
        names = list(ORIGINAL.keys())
    for name in names:
        test_problem(name, iters, interp_iters)


if __name__ == "__main__":
    main()
