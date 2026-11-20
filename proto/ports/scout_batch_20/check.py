#!/usr/bin/env python3
"""Check the batch-20 scout ports against the original submissions.

Every port is built by `proto/cpc` from a temporary copy and compared with
the submission named on its `// original:` line on small random inputs, then
timed on one largest-size input.

    python3 proto/ports/scout_batch_20/check.py            # every port, 300 inputs each
    python3 proto/ports/scout_batch_20/check.py dpz 50     # one port, 50 inputs
"""
import glob, os, random, re, shutil, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")
R = random.randint


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_btoi19p2(big):
    r, c, m = (500, 500, 5000) if big else (R(1, 6), R(1, 6), R(1, 8))
    wall = 0.05 if big else random.random() * 0.5
    grid = [("".join("#" if random.random() < wall else "." for _ in range(c)),) for _ in range(r)]
    sig = "?" * m if big else "".join(random.choice("NESW?") for _ in range(m))
    return lines((r, c, m), *grid, (sig,))


def gen_cco17p4(big):
    n = 500 if big else R(1, 8)
    return lines((n,), [R(1, 50 if big else R(1, 9)) for _ in range(n)])


def gen_ccc19s4(big):
    n = 10**6 if big else R(1, 12)
    k = 1000 if big else R(1, n)
    return lines((n, k), [R(1, 10**9 if big else 9) for _ in range(n)])


def gen_cco23p1(big):
    # The statement promises neighbouring sums differ by at most 1 (the
    # original asserts it), so walk; most walks are still unsatisfiable.
    n = 10**6 if big else R(1, 12)
    k = 1000 if big else R(1, n)
    if big or random.random() < 0.6:            # windows of a real string
        bits = [R(0, 1) for _ in range(n)]
        sums, s = [], sum(bits[:k])
        for i in range(n - k + 1):
            sums.append(s)
            if i + k < n:
                s += bits[i + k] - bits[i]
    else:
        sums = [R(0, k)]
        for _ in range(n - k):
            sums.append(min(k, max(0, sums[-1] + R(-1, 1))))
    return lines((n, k), sums)


def gen_btoi17p5(big):
    n, m = (10**9, 10**9) if big else random.choice([(a, b) for a in range(1, 6) for b in range(1, 6) if a * b <= 12])
    if big:
        cells = {(R(1, n), R(1, m)) for _ in range(100000)}
        return lines((n, m, len(cells)), *[("+-"[(y + (x % 3 == 0)) % 2], x, y) for x, y in cells])
    k = R(0, min(n * m, 6))
    cells = random.sample([(x, y) for x in range(1, n + 1) for y in range(1, m + 1)], k)
    mode = R(0, 2)                              # random, row-alternating, checkerboard
    sign = lambda x, y: random.choice("+-") if mode == 0 else "+-"[(y + (x % 2) * (mode == 2)) % 2]
    return lines((n, m, k), *[(sign(x, y), x, y) for x, y in cells])


def ref_btoi17p5(data):
    """Every grid, tried. The original is not used on small inputs: it answers
    5 instead of 6 on `5 5 5 / - 1 1 / + 2 1 / - 5 2 / - 5 4 / + 4 2`, where
    rows and columns can each alternate but neither checkerboard fits."""
    tok = data.split()
    n, m, k = map(int, tok[:3])
    fixed = [(tok[3 + 3 * i] == "+", int(tok[4 + 3 * i]) - 1, int(tok[5 + 3 * i]) - 1) for i in range(k)]
    total = 0
    for g in range(1 << (n * m)):
        bit = lambda x, y: g >> (x * m + y) & 1
        if all(bit(x, y) == plus for plus, x, y in fixed) and all(
            bit(x, y) + bit(x + 1, y) + bit(x, y + 1) + bit(x + 1, y + 1) == 2
            for x in range(n - 1) for y in range(m - 1)
        ):
            total += 1
    return [str(total % (10**9 + 7))]


def gen_noi02p4(big):
    # An answer must exist: no two cavemen may meet on an unbounded line, and
    # nobody walks further than the 10^6 caves the answer is promised to fit in.
    while True:
        n = 15 if big else R(1, 6)
        caves = random.sample(range(1, 101 if big else 13), n)
        men = [(c, R(1, 100 if big else 9), R(1, 5000 if big else 12)) for c in caves]
        if not any(
            a[0] + a[1] * x == b[0] + b[1] * x
            for a in men for b in men if a < b for x in range(1, min(a[2], b[2]) + 1)
        ):
            return lines((n,), *men)


def gen_ceoi18p5(big):
    return lines((735134400 if big else R(2, 5000),))


def gen_btoi19p4(big):
    if big:                                     # feasible, so the knapsack runs in full
        return lines((300, 300, 100), [R(100, 150) for _ in range(300)], [R(1, 300) for _ in range(300)])
    n, m, k = R(1, 4), R(1, 6), R(1, 4)
    need = [R(k if random.random() < 0.9 else 1, 6) for _ in range(n)]
    return lines((n, m, k), need, [R(1, 6) for _ in range(m)])


def gen_ioi19p1(big):
    n = 100000 if big else R(1, 6)
    sizes = [R(1, n if big else R(1, n)) for _ in range(n)]
    shoes = sizes + [-s for s in sizes]
    random.shuffle(shoes)
    return lines((n,), shoes)


def gen_ioi09p8(big):
    n = 500000 if big else R(1, 7)
    top = 500001 if big else 12
    days = 500000 if big else R(1, 5)
    up = R(1, 10 if big else 4)
    # No two fairs share a day and a location: the port adds both profits
    # there and the original keeps one.
    spots = set()
    while len(spots) < n:
        spots.add((R(1, days), R(1, top)))
    fairs = [(t, l, R(1, 4000 if big else 30)) for t, l in sorted(spots, key=lambda _: random.random())]
    return lines((n, up, R(up, 10 if big else 6), R(1, top)), *fairs)


# IOI 2019 Arranging Shoes is graded through a function, so the original has
# no `main`; this one reads the port's input format. It avoids `long` and
# `endl`, which the original redefines.
MAIN = {
    "ioi19p1": """
int main() { int n; scanf("%d", &n); std::vector<int> a(2 * n); for (auto& x : a) scanf("%d", &x);
    printf("%s ", std::to_string(count_swaps(a)).c_str()); }
""",
}


def run(exe, data, limit=30):
    # Some originals keep megabytes of tables on the stack.
    cmd = "ulimit -s unlimited 2>/dev/null; exec " + exe
    return subprocess.run(["bash", "-c", cmd], input=data, capture_output=True, text=True, timeout=limit).stdout.split()


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()]
    count = next((int(a) for a in sys.argv[1:] if a.isdigit()), 300)
    ports = sorted(glob.glob(os.path.join(HERE, "*.cp")))
    tmp = tempfile.mkdtemp()
    failed = False
    for port in ports:
        name = os.path.basename(port)[:-3]
        if names and name not in names:
            continue
        original = re.search(r"// original: (\S+)", open(port).read()).group(1)
        copy = os.path.join(tmp, name + ".cp")
        shutil.copy(port, copy)
        subprocess.run([sys.executable, CP, "build", copy], check=True, capture_output=True)
        mine, theirs = os.path.join(tmp, name), os.path.join(tmp, name + "_orig")
        subprocess.run(["g++", "-O2", "-std=c++17", "-w", "-o", mine, copy[:-3] + ".cpp"], check=True)
        source = os.path.join(tmp, name + "_orig.cpp")
        open(source, "w").write(open(os.path.join(ROOT, original)).read() + MAIN.get(name, ""))
        subprocess.run(["g++", "-O2", "-std=c++17", "-w", "-o", theirs, source], check=True)
        gen = globals()["gen_" + name]
        random.seed(20)
        ref = globals().get("ref_" + name) or (lambda data: run(theirs, data))
        bad = None
        for _ in range(count):
            data = gen(False)
            if run(mine, data) != ref(data):
                bad = data
                break
        big = gen(True)
        t0 = time.time(); a = run(mine, big, 120); t1 = time.time(); b = run(theirs, big, 120); t2 = time.time()
        if bad is None and a != b:
            bad = "(the largest input)"
        failed |= bad is not None
        print(f"{name:10} {'FAIL' if bad else 'ok':4} {count} inputs; largest: port {t1 - t0:.2f}s, original {t2 - t1:.2f}s")
        if bad:
            print(bad)
    shutil.rmtree(tmp)
    sys.exit(1 if failed else 0)


main()
