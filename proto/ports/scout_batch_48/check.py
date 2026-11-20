#!/usr/bin/env python3
"""Differential check for scout batch 48.

Builds every today/*.cp port with proto/cp.py (on temporary copies, because the
build stamps its input), compiles the original accepted C++ from
corpus/submissions/, and compares both on random small and medium inputs.

    python3 proto/ports/scout_batch_48/check.py [problem ...]
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CORPUS = os.path.join(ROOT, "corpus", "submissions")


def compile_cpp(src, out, flags=()):
    subprocess.run(["g++", "-std=c++17", "-O2", "-w", *flags, src, "-o", out], check=True)


def compile_cp(name, td):
    cp = os.path.join(td, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), cp)
    subprocess.run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "build", cp],
                   check=True, stdout=subprocess.DEVNULL)
    out = os.path.join(td, name + "_cp")
    compile_cpp(os.path.join(td, name + ".cpp"), out)
    return out


def run(binary, inp, ok_fail=False):
    p = subprocess.run([binary], input=inp, capture_output=True, text=True, timeout=20)
    if p.returncode != 0:
        if ok_fail:
            return None
        raise RuntimeError(f"{binary} exited {p.returncode} on:\n{inp[:400]}\n{p.stderr[:400]}")
    return p.stdout.split()


def same(orig, port, inp):
    a, b = run(orig, inp), run(port, inp)
    assert a == b, f"mismatch on:\n{inp[:600]}\noriginal: {a[:40]}\nport:     {b[:40]}"


def join(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_ceoi19pp1(rng, big):
    n, m = (rng.randint(1, 60), rng.randint(1, 60)) if big else (rng.randint(1, 6), rng.randint(1, 6))
    hi = rng.choice([3, 10, 10**9])
    return join([n, m], [rng.randint(-hi, hi) for _ in range(n)], [rng.randint(-hi, hi) for _ in range(m)])


def gen_ceoi19pp2(rng, big):
    q = rng.randint(1, 2000 if big else 12)
    hi = rng.choice([5, 50, 10**9])
    return join([q], *[[rng.randint(0, hi)] for _ in range(q)])


def gen_egoi22p1(rng, big):
    rows = []
    t = rng.randint(1, 5)
    for _ in range(t):
        n = rng.randint(1, 50 if big else 6)
        hi = rng.choice([1, 3, 10**9 if n < 20 else 3])
        rows += [[n], [rng.randint(0, hi) for _ in range(n)]]
    return join([t], *rows)


def gen_egoi22p5(rng, big):
    n, q = (rng.randint(1, 300), rng.randint(0, 300)) if big else (rng.randint(1, 6), rng.randint(0, 6))
    hi = rng.choice([5, 10**9])  # the original stores counts in 32-bit ints
    return join([n, q], [rng.randint(0, hi) for _ in range(n)],
                *[[rng.randint(0, min(hi, 10**6)), rng.randint(0, n)] for _ in range(q)])


def gen_egoi23p1(rng, big):
    n, q = (rng.randint(1, 300), rng.randint(1, 300)) if big else (rng.randint(1, 6), rng.randint(1, 8))
    hi = rng.choice([4, 12, 10**6])
    rows = [[n], [rng.randint(1, hi) for _ in range(n)], [q]]
    for _ in range(q):
        if rng.random() < 0.4:
            rows.append(["INFLATION", rng.randint(1, hi)])
        else:
            rows.append(["SET", rng.randint(1, 3 * hi), rng.randint(1, 3 * hi)])
    return join(*rows)


def gen_egoi23p5(rng, big):
    n = rng.randint(2, 120 if big else 7)
    rows = [[n]]
    for i in range(1, n):
        rows.append(rng.sample(range(i), i))
    return join(*rows)


def gen_egoi23p6(rng, big):
    n = rng.randint(1, 30 if big else 6)  # the original keeps its table on the stack
    m = rng.randint(1, n)
    a = [rng.randint(0, rng.choice([3, 10**9])) for _ in range(n)]
    k = rng.randint(1, max(1, sum(sorted(a)[-m:]) + rng.randint(0, 2)))
    return join([n, m, k], a)


def gen_btoi06p5(rng, big):
    n = rng.randint(2, 40 if big else 5)
    out, e = [], 0
    for _ in range(rng.randint(1, 400 if big else 12)):
        r = rng.random()
        if r < 0.45:
            out.append(rng.choice([s for s in range(n) if s != e]))
        elif r < 0.6:
            out += [e, e, rng.randint(0, n - 1)]
        elif r < 0.75:
            e2 = rng.choice([s for s in range(n) if s != e])
            out += [e, e2, 0]
            e = e2
        else:
            out += [e, rng.choice([s for s in range(n) if s != e]), rng.randint(1, n - 1)]
    return join([n, len(out)], out)


def stdin_test(gen):
    def test(orig, port, rng):
        for i in range(400):
            same(orig, port, gen(rng, i % 4 == 3))
    return test


def test_egoi23p8(orig, port, rng):
    # phase 1 on a random reveal order, then phase 2 on the labels the original
    # produced with two houses erased. The original (91/100) aborts or reads
    # past its candidate list on some erasures; those cases are skipped.
    done = 0
    for i in range(300):
        n = rng.randint(13, 400 if i % 4 == 3 else 20)
        order = rng.sample(range(n), n)
        inp = join([1], [n], *[[h] for h in order[:-1]])
        a, b = run(orig, inp), run(port, inp)
        assert a == b, f"phase 1 mismatch on:\n{inp}"
        label = [0] * n
        for h, v in zip(order, a[1:]):
            label[h] = int(v)
        for _ in range(4):
            cut = label[:]
            for h in rng.sample(order[:-1], 2):
                cut[h] = 0
            inp2 = join([2], [n], cut)
            flags = [v for v in cut if v >= 11]
            coded = sum(2 <= v <= 10 for v in cut)
            if cut.count(1) == n - 13 and len(flags) == 1 and (flags[0] - 11) * 2 + 1 >= coded:
                continue
            want = run(orig, inp2, ok_fail=True)
            if want is None:
                continue
            got = run(port, inp2)
            assert want == got, f"phase 2 mismatch on:\n{inp2}\n{want} vs {got}"
            done += 1
    assert done > 600, done


HARNESS_EGOI22P8 = r"""
#define main original_main
#include "%s"
#undef main
int main() {
    int t; scanf("%%d", &t);
    while (t--) {
        int kind; scanf("%%d", &kind);
        if (kind == 1) {
            int n; scanf("%%d", &n);
            auto pat = BuildPattern(n);
            printf("%%d\n", (int)pat.size());
            for (auto& [p, q] : pat) printf("%%d %%d %%d %%d\n", p.first, p.second, q.first, q.second);
        } else {
            int k; scanf("%%d", &k);
            vector<pair<pair<int, int>, pair<int, int>>> pat(k);
            for (auto& [p, q] : pat) scanf("%%d %%d %%d %%d", &p.first, &p.second, &q.first, &q.second);
            printf("%%d\n", GetCardNumber(pat));
        }
    }
}
"""


def test_egoi22p8(orig, port, rng):
    def turn(p):
        return (2 - p[1], p[0])
    for i in range(120):
        ns = [rng.randint(1, rng.choice([30, 5000, 67_000_000])) for _ in range(5)]
        inp = join([len(ns)], *[[1, n] for n in ns])
        a, b = run(orig, inp), run(port, inp)
        assert a == b, f"BuildPattern mismatch on {ns}"
        rows, pos = [], 0
        for n in ns:
            k = int(a[pos]); pos += 1
            segs = []
            for _ in range(k):
                x1, y1, x2, y2 = map(int, a[pos:pos + 4]); pos += 4
                segs.append(((x1, y1), (x2, y2)))
            rng.shuffle(segs)
            for _ in range(rng.randint(0, 3)):
                segs = [(turn(p), turn(q)) for p, q in segs]
            segs = [(q, p) if rng.random() < 0.5 else (p, q) for p, q in segs]
            rows.append([2, k])
            rows += [[*p, *q] for p, q in segs]
        inp2 = join([len(ns)], *rows)
        a2, b2 = run(orig, inp2), run(port, inp2)
        assert a2 == b2 == [str(n) for n in ns], f"GetCardNumber mismatch on {ns}: {a2} {b2}"


def build_egoi22p8(td):
    h = os.path.join(td, "egoi22p8_harness.cpp")
    with open(h, "w") as f:
        f.write(HARNESS_EGOI22P8 % os.path.join(CORPUS, "5202737.cpp"))
    out = os.path.join(td, "egoi22p8_orig")
    compile_cpp(h, out)
    return out


PROBLEMS = [
    ("btoi06p5", "4156310", stdin_test(gen_btoi06p5), None),
    ("ceoi19pp1", "5343174", stdin_test(gen_ceoi19pp1), None),
    ("ceoi19pp2", "5343197", stdin_test(gen_ceoi19pp2), None),
    ("egoi22p1", "5198974", stdin_test(gen_egoi22p1), None),
    ("egoi22p5", "5203693", stdin_test(gen_egoi22p5), None),
    ("egoi22p8", "5202737", test_egoi22p8, build_egoi22p8),
    ("egoi23p1", "5678049", stdin_test(gen_egoi23p1), None),
    ("egoi23p5", "5678053", stdin_test(gen_egoi23p5), None),
    ("egoi23p6", "5678056", stdin_test(gen_egoi23p6), None),
    ("egoi23p8", "5659039", test_egoi23p8, None),
]


def main():
    want = set(sys.argv[1:])
    failed = []
    with tempfile.TemporaryDirectory() as td:
        for name, sub, test, build in PROBLEMS:
            if want and name not in want:
                continue
            try:
                if build:
                    orig = build(td)
                else:
                    orig = os.path.join(td, name + "_orig")
                    compile_cpp(os.path.join(CORPUS, sub + ".cpp"), orig)
                port = compile_cp(name, td)
                test(orig, port, random.Random(48))
                print(f"ok   {name}")
            except Exception as e:  # keep going so one failure shows all results
                failed.append(name)
                print(f"FAIL {name}: {str(e)[:1500]}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
