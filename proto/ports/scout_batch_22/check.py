#!/usr/bin/env python3
"""Check the batch-22 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs and must print the same output.

Four problems have many right answers (ioi08p1, noi09p2, ccc01s5, ccc99s5):
there both outputs are judged against the statement instead of compared.
Two originals are IOI functions with no `main` (ioi16p4, ioi21p4): a `main`
that reads the format in the port's header comment is appended before
compiling.

    python3 proto/ports/scout_batch_22/check.py                # every port, 60 inputs each
    python3 proto/ports/scout_batch_22/check.py noi18p3 200    # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint

ORIGINAL = {
    "ioi08p1": 5034409, "noi09p2": 5108899, "cco19p6": 3617414, "noi14p1": 3595714,
    "ioi21p4": 4698075, "noi21p4": 3861375, "ccc01s5": 5025769, "ccc99s5": 5515588,
    "ioi16p4": 4686585, "noi18p3": 6513115,
}

MAIN = {
    "ioi16p4": """
int main() {
    string s; int k; cin >> s >> k;
    vector<int> c(k);
    for (auto& x : c) cin >> x;
    cout << solve_puzzle(s, c) << endl;
}
""",
    "ioi21p4": """
int main() {
    int n, q; string a, b; cin >> n >> q >> a >> b;
    init(a, b);
    while (q--) { int x, y; cin >> x >> y; cout << get_distance(x, y) << endl; }
}
""",
}

def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def word(n, letters="ab"):
    return "".join(random.choice(letters) for _ in range(n))


def gen_ioi08p1():
    words = {word(R(1, 5), random.choice(["ab", "abc"])) for _ in range(R(1, 8))}
    return lines([(len(words),)] + [(w,) for w in words])


def gen_cco19p6():
    m = R(1, 5)
    words = {word(R(1, m), "01") for _ in range(R(1, 6))}
    return lines([(len(words), m)] + [(w,) for w in words])


def gen_noi14p1():
    top = random.choice([7, 1000, 10 ** 9])
    n = R(1, 6)
    return lines([(n, R(0, top))] + [(random.choice(["AND", "OR", "XOR"]), R(0, top)) for _ in range(n)])


def gen_ioi21p4():
    n, q = R(1, 10), R(1, 8)
    a = word(n, "ACT")
    b = "".join(random.sample(a, n)) if R(0, 1) else word(n, "ACT")
    qs = [sorted((R(0, n - 1), R(0, n - 1))) for _ in range(q)]
    return lines([(n, q), (a,), (b,)] + qs)


def gen_noi21p4():
    M = (1 << 64) - 1
    n, q = R(1, 12), R(1, 8)
    a1, a2 = (R(0, M) if R(0, 3) else R(0, 9) for _ in range(2))
    k1, k2 = a1, a2
    dictionary = []
    for _ in range(n):
        w = 0
        for _ in range(256):
            k3, k4 = k1, k2
            k1 = k4
            k3 ^= (k3 << 23) & M
            k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26)
            w = w << 1 | ((k2 + k4) & M) >> 32 & 1
        dictionary.append(w)
    rows = [(n, q, a1, a2)]
    for _ in range(q):
        w = random.choice(dictionary)
        for _ in range(R(0, 18)):
            w ^= 1 << R(0, 255)
        if R(0, 1):
            w ^= (1 << 256) - 1
        rows.append(("%064X" % w, R(0, 15)))
    return lines(rows)


def gen_ccc01s5():
    n, m = R(1, 4), R(2, 6)
    return lines([(m,), (n,)] + [(word(R(1, 3)),) for _ in range(2 * n)])


def gen_ccc99s5():
    rows = [(1,)]
    while True:
        x, y = R(1, 10 ** R(1, 4)), R(1, 10 ** R(1, 4))
        nums = [str(x), str(y), str(x + y)]
        digits = sorted(set("".join(nums)))
        letters = random.sample("ABCDEFGHIJKLMNOPQRSTUVWXYZ", len(digits))
        code = dict(zip(digits, letters))
        return lines(rows + [("".join(code[d] for d in s),) for s in nums])


def gen_ioi16p4():
    k = R(1, 4)
    clues = [R(1, 3) for _ in range(k)]
    cells = []
    for i, c in enumerate(clues):
        cells += ["_"] * R(0 if i == 0 else 1, 2) + ["X"] * c
    cells += ["_"] * R(0, 2)
    row = "".join(c if R(0, 3) == 0 else "." for c in cells)
    return lines([(row,), (k,), clues])


def gen_noi18p3():
    letters = random.choice(["ab", "abc"])
    s = word(R(1, 12), letters)
    rows = [(s,), (0,)]
    for _ in range(R(1, 6)):
        l = R(1, len(s))
        rows.append((word(R(1, 8), letters), l, R(l, len(s))))
    rows[1] = (len(rows) - 2,)
    return lines(rows)


def gen_noi09p2():
    rows = [(1,)]
    n = R(1, 8)
    big = R(0, 5) == 0
    rows.append((n, R(10 ** 6, 3 * 10 ** 6) if big else R(1, 12), R(4, 10) if big else R(1, 3)))
    return lines(rows + [(word(R(1, 5), "abc"),) for _ in range(n)])


def judge_noi09p2(inp, out):
    """The cost on the first line, and the lines must be the sentences in order at that cost."""
    toks = inp.split()
    l, p, words = int(toks[2]), int(toks[3]), toks[4:]
    rows = out.splitlines()
    if rows[-1] != "-" * 20:
        return (None, False)
    if rows[0] == "Too hard to arrange":
        return ("hard", len(rows) == 2)
    poem = rows[1:-1]
    ok = " ".join(poem).split() == words and all(r == " ".join(r.split()) for r in poem)
    return (int(rows[0]), ok and sum(abs(len(r) - l) ** p for r in poem) == int(rows[0]))


def judge_ioi08p1(inp, out):
    """The first line is the number of operations; the operations must print every word."""
    want = set(inp.split()[1:])
    toks = out.split()
    ops, line, printed = toks[1:], [], set()
    for op in ops:
        if op == "P":
            printed.add("".join(line))
        elif op == "-":
            line.pop()
        else:
            line.append(op)
    return (int(toks[0]), int(toks[0]) == len(ops) and printed == want)


def judge_ccc01s5(inp, out):
    toks = inp.split()
    m, n = int(toks[0]), int(toks[1])
    a, b = toks[2:2 + n], toks[2 + n:]
    if out.strip() == "No solution.":
        return ("none", True)
    seq = [int(x) for x in out.split()]
    k, seq = seq[0], seq[1:]
    ok = 0 < k < m and len(seq) == k and "".join(a[i - 1] for i in seq) == "".join(b[i - 1] for i in seq)
    return ("some", ok)


def judge_ccc99s5(inp, out):
    words, nums = inp.split()[1:], out.split()
    code = {}
    ok = len(nums) == 3
    for w, x in zip(words, nums):
        ok = ok and len(w) == len(x) and x[0] != "0"
        for c, d in zip(w, x):
            ok = ok and code.setdefault(c, d) == d
    ok = ok and len(set(code.values())) == len(code) and int(nums[0]) + int(nums[1]) == int(nums[2])
    return ("solved", ok)


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
    with tempfile.TemporaryDirectory() as tmp:
        exe, port = compile_original(name, tmp), os.path.join(HERE, name + ".cp")
        for case in range(count):
            inp = gen()
            # Some originals keep megabytes of arrays on the stack.
            want = run(["bash", "-c", f"ulimit -s unlimited; exec {exe}"], inp)
            got = run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", port], inp)
            if judge:
                same = not got.startswith("<exit") and judge(inp, got) == judge(inp, want) and judge(inp, got)[1]
            else:
                same = got == want
            if not same:
                print(f"FAIL {name} case {case}\n--- input\n{inp}--- original\n{want}--- port\n{got}")
                return False
    print(f"ok   {name}: {count} inputs")
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(ORIGINAL)
    count = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(22)
    sys.exit(0 if all([check(n, count) for n in names]) else 1)


if __name__ == "__main__":
    main()
