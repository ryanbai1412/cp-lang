#!/usr/bin/env python3
"""Check the batch-29 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs and must print the same output.

Three problems are judged instead of compared: ioi09p2 and noi20p4 accept
any best answer, so both outputs are checked against the statement, and
sdoi17r1p6 prints rounded decimals, compared to within 2e-5.
One original is a set of IOI grader functions with no `main` (ioi15p4): a
`main` that reads the format in the port's header comment is appended.

    python3 proto/ports/scout_batch_29/check.py                # every port, 60 inputs each
    python3 proto/ports/scout_batch_29/check.py noip20p1 200   # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint

ORIGINAL = {
    "coci25c3p5": 7730259, "noip19p4": 5147491, "noimps22p4": 4547602, "shoi16p4": 5635192,
    "noip20p1": 5445672, "noip19p5": 5148191, "ioi15p4": 5723351, "ioi09p2": 5102705,
    "noi20p4": 3890337, "sdoi17r1p6": 5461671,
}

MAIN = {
    "ioi15p4": """
int main() {
    int n; cin >> n;
    vector<int> x(n), y(n);
    for (auto& a : x) cin >> a;
    for (auto& a : y) cin >> a;
    cout << init(n, x.data(), y.data()) << endl;
    int m; cin >> m;
    while (m--) { int k, p, v; cin >> k >> p >> v; cout << (k == 1 ? updateX(p, v) : updateY(p, v)) << endl; }
}
""",
}

# The interpreter spends about two seconds building this port's tables.
SLOW = {"noimps22p4": 4}


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def gen_coci25c3p5():
    n, top = R(1, 7), random.choice([6, 40, 10**9])
    return lines([(n,), [R(1, top) for _ in range(n)], [R(1, top) for _ in range(n)]])


def gen_noip19p4():
    n, m, top = R(1, 6), R(1, 5), random.choice([1, 3, 998244352])
    return lines([(n, m)] + [[R(0, top) for _ in range(m)] for _ in range(n)])


PRIMES = [p for p in range(2, 2001) if all(p % d for d in range(2, int(p ** 0.5) + 1))]


def gen_noimps22p4():
    pool = random.sample(PRIMES[:20], 4) + random.sample(PRIMES, 2)
    def value():
        v = 1
        for _ in range(R(0, 3)):
            p = random.choice(pool)
            if v * p <= 2000:
                v *= p
        return v
    n, q = R(1, 9), R(1, 4)
    rows = [(n,), [value() for _ in range(n)], (q,)]
    for _ in range(q):
        ps = [random.choice(pool) for _ in range(R(1, 4))]
        rows += [(len(ps),), ps]
    return lines(rows)


def gen_shoi16p4():
    n = R(2, 6)
    rows = [(n,)]
    for _ in range(n - 1):
        m = R(1, 4)
        rows.append([m] + [v for _ in range(m) for v in random.sample(range(1, n + 1), 2)])
    return lines(rows)


def gen_noip20p1():
    if R(0, 4) == 0:
        # A chain of 11 splits by 3, 4 and 5: denominators near 60^11.
        degs = [random.choice([3, 4, 5]) for _ in range(11)]
        n, rows, sink = 12 + sum(d - 1 for d in degs), [], 13
        for i, d in enumerate(degs):
            rows.append([d, i + 2] + list(range(sink, sink + d - 1)))
            sink += d - 1
        rows += [[0]] * (n - 11)
        return lines([(n, 1)] + rows)
    n = R(2, 12)
    m = R(1, n - 1)
    out = [[] for _ in range(n + 1)]
    for v in range(m + 1, n + 1):       # every junction past the sources gets a pipe in
        u = random.choice([u for u in range(1, v) if len(out[u]) < 5] or [0])
        if u == 0:
            return gen_noip20p1()
        out[u].append(v)
    for u in range(1, n):
        for v in random.sample(range(max(u, m) + 1, n + 1), min(R(0, 3), n - max(u, m))):
            if v not in out[u] and len(out[u]) < 5:
                out[u].append(v)
    return lines([(n, m)] + [[len(o)] + o for o in out[1:]])


def gen_noip19p5():
    n, top = R(1, 30), random.choice([5, 100, 10**9])
    if R(0, 1):
        return lines([(n, 0), [R(1, top) for _ in range(n)]])
    cuts = sorted(set(random.sample(range(1, n + 1), R(1, min(n, 4))) + [n]))
    rows = [(n, 1), [R(0, 2**30 - 1) for _ in range(5)] + [len(cuts)]]
    for p in cuts:
        l = R(1, top)
        rows.append((p, l, R(l, top)))
    return lines(rows)


def gen_ioi15p4():
    n, m = R(1, 9), R(0, 7)
    xs, ys = random.choice([[1, 1, 2], [1, 2, 3], [1, 10**9]]), random.choice([[1, 2, 3, 9], [1, 10**9, 10**9 - 1]])
    rows = [(n,), [random.choice(xs) for _ in range(n)], [random.choice(ys) for _ in range(n)], (m,)]
    for _ in range(m):
        kind = R(1, 2)
        rows.append((kind, R(0, n - 1), random.choice(xs if kind == 1 else ys)))
    return lines(rows)


def gen_ioi09p2():
    n, top = R(1, 9), random.choice([4, 20, 20000])
    ask = [(R(1, top), R(1, top)) for _ in range(n)]
    # The original hires one candidate even when nobody is affordable; keep one affordable.
    return lines([(n, R(min(s for s, _ in ask), top * 6))] + ask)


def gen_noi20p4():
    rows = [(R(1, 4),)]
    for _ in range(rows[0][0]):
        n, k = R(1, 7), R(1, 9)
        m = R(max(1, n - 2), n + 2)
        while m * k < n:
            m += 1
        cuts = sorted(random.sample(range(1, m * k), n - 1))
        rows += [(n, m, k), [b - a for a, b in zip([0] + cuts, cuts + [m * k])]]
    return lines(rows)


def gen_sdoi17r1p6():
    n, q = R(2, 9), R(1, 9)
    rows = [(n, q), random.sample(range(0, 30), n), [R(0, 30) for _ in range(n)]]
    for _ in range(q):
        kind = R(1, 3)
        l = R(1, n - 1) if kind == 1 else R(1, n)
        r = R(l + 1, n) if kind == 1 else R(l, n)
        rows.append((kind, l, r) if kind == 1 else (kind, l, r, R(-6, 6), R(-6, 6)))
    # A slope needs two different x in its range (the statement promises it), and an
    # edit of part of the array can make two x equal: edit the whole array instead.
    rows = [r if len(r) != 5 else (r[0], 1, n, r[3], r[4]) for r in rows]
    return lines(rows)


def judge_ioi09p2(inp, out):
    """The hires must fit the budget; returns their count and bill."""
    toks = list(map(int, inp.split()))
    w, ask = toks[1], list(zip(toks[2::2], toks[3::2]))
    nums = list(map(int, out.split()))
    hired = nums[1:]
    ok = nums[0] == len(hired) == len(set(hired)) and all(1 <= i <= len(ask) for i in hired)
    if not ok or not hired:
        return ((len(hired), 0), ok)
    bill = max(Fraction(ask[i - 1][0], ask[i - 1][1]) for i in hired) * sum(ask[i - 1][1] for i in hired)
    return ((len(hired), bill), bill <= w)


def judge_noi20p4(inp, out):
    """Per case: -1, or m dishes of k grams that use every ingredient up."""
    toks, rows, verdict, ok = list(map(int, inp.split())), out.splitlines(), [], True
    at = 1
    for _ in range(toks[0]):
        n, m, k = toks[at:at + 3]
        left = [0] + toks[at + 3:at + 3 + n]
        at += 3 + n
        if rows and rows[0] == "-1":
            rows, verdict = rows[1:], verdict + [False]
            continue
        dishes, rows, verdict = rows[:m], rows[m:], verdict + [True]
        ok = ok and len(dishes) == m
        for d in dishes:
            v = list(map(int, d.split()))
            ok = ok and len(v) in (2, 4) and sum(v[1::2]) == k and min(v[1::2]) > 0 and len(set(v[0::2])) == len(v) // 2
            for i, g in zip(v[0::2], v[1::2]):
                left[i] -= g
        ok = ok and not any(left)
    return (verdict, ok and not rows)


def judge_sdoi17r1p6(inp, out):
    """Slopes rounded to 5 decimals: bucket them, so a last-digit tie still matches."""
    vals = [float(v) for v in out.split()]
    return (len(vals), True, vals)


def agree(a, b):
    if a[:2] != b[:2] or not a[1]:
        return False
    return len(a) < 3 or all(abs(x - y) <= 2e-5 for x, y in zip(a[2], b[2]))


def compile_original(name, tmp):
    src = open(os.path.join(ROOT, "corpus", "submissions", f"{ORIGINAL[name]}.cpp")).read() + MAIN.get(name, "")
    path = os.path.join(tmp, name + ".cpp")
    open(path, "w").write(src)
    exe = os.path.join(tmp, name)
    subprocess.run(["g++", "-std=c++17", "-O1", "-w", "-o", exe, path], check=True)
    return exe


def run(cmd, inp):
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=120)
    return r.stdout if r.returncode == 0 else f"<exit {r.returncode}> {r.stdout}{r.stderr[-400:]}"


def check(name, count):
    gen, judge = globals()["gen_" + name], globals().get("judge_" + name)
    count = max(1, count // SLOW.get(name, 1))
    with tempfile.TemporaryDirectory() as tmp:
        exe, port = compile_original(name, tmp), os.path.join(HERE, name + ".cp")
        for case in range(count):
            inp = gen()
            # Some originals keep megabytes of arrays on the stack.
            want = run(["bash", "-c", f"ulimit -s unlimited; exec {exe}"], inp)
            got = run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", port], inp)
            if judge:
                same = not got.startswith("<exit") and agree(judge(inp, got), judge(inp, want))
            else:
                same = got == want
            if not same:
                print(f"FAIL {name} case {case}\n--- input\n{inp}--- original\n{want}--- port\n{got}")
                return False
    print(f"ok   {name}: {count} inputs")
    return True


def main():
    names = [a for a in sys.argv[1:] if a in ORIGINAL] or list(ORIGINAL)
    count = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(29)
    sys.exit(0 if all([check(n, count) for n in names]) else 1)


if __name__ == "__main__":
    main()
