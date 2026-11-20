#!/usr/bin/env python3
"""Check the batch-28 scout ports against the original submissions.

Each port is built to C++17 (`proto/cp.py build` on a temporary copy) and
the original is compiled from `corpus/submissions/<id>.cpp`; both get the
same random inputs, small and medium, and must print the same output.

ioi00p5 accepts any best set of post offices: there both outputs are judged
against the statement instead of compared. Two originals are IOI functions
with no `main` (ioi20p4, ioi16p6): a `main` that reads the sample-grader
format is appended before compiling.

    python3 proto/ports/scout_batch_28/check.py                # every port, 200 inputs each
    python3 proto/ports/scout_batch_28/check.py ioi16p6 1000   # one port, 1000 inputs
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint

ORIGINAL = {
    "ioi09p4": 4817803, "noip08p3": 5185022, "dwite10c3p3": 4804312, "noip21p3": 5445678,
    "ioi00p5": 4811886, "ioi04p2": 4171535, "noip10p2": 5445662, "noip05p2": 5156196,
    "ioi20p4": 4733047, "ioi16p6": 4730545,
}

MAIN = {
    "ioi20p4": """
int main() {
    int q; cin >> q;
    while (q--) {
        int k; ll x; cin >> k >> x;
        vector<ll> a(k);
        for (auto& v : a) cin >> v;
        cout << count_tastiness(x, a) << endl;
    }
}
""",
    "ioi16p6": """
int main() {
    int n, m, k; cin >> n >> m >> k;
    vector<int> r(n), c(n);
    for (int i = 0; i < n; i++) cin >> r[i] >> c[i];
    cout << take_photos(n, m, k, r, c) << endl;
}
""",
}


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def medium():
    return R(0, 9) == 0


def gen_ioi09p4():
    n, m = (R(1, 14), R(1, 14)) if medium() else (R(1, 5), R(1, 5))
    return lines([(n, m)] + [[R(1, 1000) for _ in range(m)] for _ in range(n)])


def gen_noip08p3():
    n, m = (R(2, 50), R(2, 50)) if medium() else (R(2, 5), R(2, 5))
    grid = [[R(0, 100) for _ in range(m)] for _ in range(n)]
    grid[0][0] = grid[-1][-1] = 0
    return lines([(n, m)] + grid)


def gen_dwite10c3p3():
    return lines([(R(1, 30),) for _ in range(5)])


def gen_noip21p3():
    n, top = (R(1, 60), R(1, 40)) if medium() else (R(1, 8), R(1, 12))
    return lines([(n,), sorted(R(1, top) for _ in range(n))])


def gen_ioi00p5():
    n = R(1, 60) if medium() else R(1, 9)
    spots = sorted(random.sample(range(1, random.choice([20, 100, 10000]) + n), n))
    return lines([(n, R(1, min(n, 30))), spots])


def judge_ioi00p5(inp, out):
    """The cost on the first line; the offices must be villages, increasing, and cost that much."""
    toks = [int(t) for t in inp.split()]
    m, spots = toks[1], toks[2:]
    rows = out.splitlines()
    cost, offices = int(rows[0]), [int(t) for t in rows[1].split()]
    ok = (len(offices) == m and offices == sorted(set(offices)) and set(offices) <= set(spots)
          and sum(min(abs(s - o) for o in offices) for s in spots) == cost)
    return (cost, ok)


def gen_ioi04p2():
    n, top = (R(1, 300), 1000) if medium() else (R(1, 7), random.choice([3, 10, 1000]))
    return lines([(n,)] + [(R(-top, top), R(-top, top)) for _ in range(n)])


def gen_noip10p2():
    m = R(1, 120) if medium() else R(1, 8)
    have = [0] * 4
    cards = []
    for _ in range(m):
        c = random.choice([i for i in range(4) if have[i] < 40])
        have[c] += 1
        cards.append(c + 1)
    n = sum(cards) + 1
    return lines([(n, m), [R(0, 100) for _ in range(n)], cards])


def gen_noip05p2():
    length = random.choice([R(2, 40), R(2, 40), R(10 ** 4, 10 ** 5), R(10 ** 8, 10 ** 9)])
    s = R(1, 10)
    t = R(s, 10)
    m = R(1, min(100 if medium() else 8, length - 1))
    near = R(0, 1)
    stones = set()
    while len(stones) < m:
        stones.add(R(1, min(length - 1, 60)) if near and R(0, 3) else R(1, length - 1))
    return lines([(length,), (s, t, m), random.sample(sorted(stones), m)])


def gen_ioi20p4():
    rows = [(0,)]
    for _ in range(R(1, 4)):
        k = R(1, 60) if medium() else R(1, 6)
        left = random.choice([30, 10 ** 4, 10 ** 18])
        a = [0] * k
        for i in random.sample(range(k), k):
            a[i] = R(0, (left >> i) // random.choice([1, 1, 2, 50]))
            left -= a[i] << i
        rows += [(k, random.choice([1, R(1, 5), R(1, 10 ** random.choice([2, 9, 18]))])), a]
    rows[0] = ((len(rows) - 1) // 2,)
    return lines(rows)


def gen_ioi16p6():
    n, m = (R(1, 400), R(1, 10 ** 6)) if medium() else (R(1, 9), R(1, random.choice([6, 30, 10 ** 6])))
    return lines([(n, m, R(1, n))] + [(R(0, m - 1), R(0, m - 1)) for _ in range(n)])


def compile_original(name, tmp):
    src = open(os.path.join(ROOT, "corpus", "submissions", f"{ORIGINAL[name]}.cpp")).read() + MAIN.get(name, "")
    path = os.path.join(tmp, name + "_orig.cpp")
    open(path, "w").write(src)
    exe = os.path.join(tmp, name + "_orig")
    subprocess.run(["g++", "-std=c++17", "-O1", "-w", "-o", exe, path], check=True)
    return exe


def compile_port(name, tmp):
    # `build` rewrites the first line of its source, so it gets a copy
    src = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, name + ".cp"), src)
    subprocess.run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "build", src], check=True, capture_output=True)
    exe = os.path.join(tmp, name)
    subprocess.run(["g++", "-std=c++17", "-O1", "-w", "-o", exe, os.path.join(tmp, name + ".cpp")], check=True)
    return exe


def run(exe, inp):
    # Some originals keep megabytes of arrays on the stack.
    r = subprocess.run(["bash", "-c", f"ulimit -s unlimited; exec {exe}"], input=inp, capture_output=True, text=True,
                       timeout=120)
    return r.stdout if r.returncode == 0 else f"<exit {r.returncode}> {r.stdout}{r.stderr[-400:]}"


def check(name, count):
    gen, judge = globals()["gen_" + name], globals().get("judge_" + name)
    with tempfile.TemporaryDirectory() as tmp:
        orig, port = compile_original(name, tmp), compile_port(name, tmp)
        for case in range(count):
            inp = gen()
            want, got = run(orig, inp), run(port, inp)
            if judge:
                same = not got.startswith("<exit") and judge(inp, got) == judge(inp, want) and judge(inp, got)[1]
            else:
                same = got == want
            if not same:
                print(f"FAIL {name} case {case}\n--- input\n{inp[:2000]}--- original\n{want}--- port\n{got}")
                return False
    print(f"ok   {name}: {count} inputs")
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(ORIGINAL)
    count = next((int(a) for a in sys.argv[1:] if a.isdigit()), 200)
    random.seed(28)
    sys.exit(0 if all([check(n, count) for n in names]) else 1)


if __name__ == "__main__":
    main()
