#!/usr/bin/env python3
"""Check the batch-21 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs. Integer answers must be equal. Real answers must agree
to the tolerance of the problem (`close`), because the two programs do not
print the same digits.

    python3 proto/ports/scout_batch_21/check.py              # every port, 60 inputs each
    python3 proto/ports/scout_batch_21/check.py ioi07p5 300  # one port, 300 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def close(tol):
    """Outputs agree when every token is the same text or two reals within tol
    (absolute or relative)."""
    def judge(inp, want, got):
        a, b = want.split(), got.split()
        if len(a) != len(b):
            return False
        for x, y in zip(a, b):
            if x == y:
                continue
            try:
                x, y = float(x), float(y)
            except ValueError:
                return False
            if not abs(x - y) <= tol * max(1.0, abs(x)):
                return False
        return True
    return judge


def gen_ioi07p5():
    # A small board and a small d, so that pairs at exactly d and animals on
    # the same cell are common.
    dims = R(1, 3)
    m = R(1, [0, 40, 12, 6][dims])
    n = R(1, 12)
    d = R(1, 3 * m)
    return lines([(dims, n, d, m)] + [[R(1, m) for _ in range(dims)] for _ in range(n)])


def gen_ccc14s4():
    # Panes on a small window overlap, share edges and stack tints often.
    n = R(1, 8)
    rows = [(n,), (R(1, 6),)]
    for _ in range(n):
        x0, y0 = R(0, 8), R(0, 8)
        rows.append((x0, y0, R(x0 + 1, 10), R(y0 + 1, 10), R(1, 3)))
    return lines(rows)


def gen_ioi08p6():
    # Obstacles may not overlap: cut the field into slabs and cells, and put
    # at most one obstacle strictly inside each cell. Half the cases have no
    # budget, which is the other half of the solution.
    m, n = R(1, 12), R(1, 12)
    boxes = []
    x = 1
    while x <= m:
        w = R(1, 4)
        y = 1
        while y <= n:
            h = R(1, 4)
            x1, y1 = min(m, x + w - 1), min(n, y + h - 1)
            if R(0, 2):
                a, c = R(x, x1), R(y, y1)
                boxes.append((a, c, R(a, x1), R(c, y1), R(1, 9)))
            y += h
        x += w
    random.shuffle(boxes)
    budget = R(0, 1) * R(1, 25)
    return lines([(m, n), (budget,), (len(boxes),)] + boxes)


def gen_sac22cc4jp3():
    # General position on purpose: no two points in the same direction and
    # no two at the same gap from the aim, because an exact tie is decided
    # by the last bit of atan2 and the two programs need not round alike.
    while True:
        n, aim = R(1, 8), R(0, 359)
        pts = [(R(-9, 9), R(-9, 9)) for _ in range(n)]
        if any(p == (0, 0) for p in pts):
            continue
        import math
        gaps = []
        for x, y in pts:
            g = (aim - math.degrees(math.atan2(y, x))) % 360
            gaps.append(min(g, 360 - g))
        gaps.sort()
        if all(b - a > 1e-6 for a, b in zip(gaps, gaps[1:])):
            return lines([(n, aim)] + pts)


def gen_cmt21peter2():
    n = R(1, 9)
    return lines([(n,)] + [(R(-50, 50), R(-50, 50)) for _ in range(n)])


def gen_sdoi17r2p1():
    rows = [(R(1, 3),)]
    for _ in range(rows[0][0]):
        m, n = R(1, 6), R(1, 7)
        rows.append((m, n))
        for _ in range(10):
            lo = R(0, n * (m - 1))
            rows.append((lo, R(lo, n * (m - 1))))
    return lines(rows)


def gen_noi07p2():
    n = R(1, 8)
    rows = [(n, R(1, 100))]
    for _ in range(n):
        rows.append(("%.2f" % (R(50, 1000) / 100), "%.2f" % (R(50, 1000) / 100), "%.1f" % (R(1, 100) / 10)))
    return lines(rows)


def gen_dpj():
    n = R(1, 9)
    return lines([(n,), [R(1, 3) for _ in range(n)]])


def gen_apio23p1():
    # Sparse on purpose, so that h is often unreachable or only reachable
    # through itself; kinds 0 and 2 are common so resets and halvings matter.
    n = R(2, 7)
    m = R(0, 8)
    roads = []
    for _ in range(m):
        u = R(0, n - 1)
        v = random.choice([x for x in range(n) if x != u])
        roads.append((u, v, R(1, 20)))
    return lines([(n, m, R(1, 4), R(1, n - 1)), [R(0, 2) for _ in range(n)]] + roads)


def gen_nccc10s4():
    # Few lines through a tiny lattice: parallel lines, repeated lines and
    # three lines through one point are all common, which is where the
    # fractions have to compare exactly.
    n = R(1, 5)
    rows = [(n,)]
    for _ in range(n):
        x1, y1 = R(-3, 3), R(-3, 3)
        x2, y2 = random.choice([(x, y) for x in range(-3, 4) for y in range(-3, 4) if (x, y) != (x1, y1)])
        rows.append((x1, y1, x2, y2))
    return lines(rows)


# The original of apio23p1 is a grader function; this gives it the input
# format the port reads.
MAIN_APIO23P1 = r"""
int main() {
    int n, m, k, h;
    scanf("%d %d %d %d", &n, &m, &k, &h);
    vector<int> arr(n), u(m), v(m), w(m);
    for (auto& a : arr) scanf("%d", &a);
    for (int i = 0; i < m; i++) scanf("%d %d %d", &u[i], &v[i], &w[i]);
    double ans = solve(n, m, k, h, u, v, w, arr);
    if (ans < 0) printf("-1\n"); else printf("%.6f\n", ans);
}
"""
MAINS = {"apio23p1": MAIN_APIO23P1}


PORTS = {
    "nccc10s4": ("4225209", gen_nccc10s4, None),
    "sac22cc4jp3": ("4416076", gen_sac22cc4jp3, None),
    "cmt21peter2": ("4041223", gen_cmt21peter2, close(0.011)),     # two decimals, rounded apart at worst
    "apio23p1": ("5593318", gen_apio23p1, close(1e-6)),
    "sdoi17r2p1": ("5481972", gen_sdoi17r2p1, close(2e-6)),
    "noi07p2": ("3767248", gen_noi07p2, close(2e-3)),
    "dpj": ("4131649", gen_dpj, close(1e-8)),
    "ccc14s4": ("5672281", gen_ccc14s4, None),
    "ioi08p6": ("5637805", gen_ioi08p6, None),
    "ioi07p5": ("3505539", gen_ioi07p5, None),
}


def big_stack():
    # Several originals keep large arrays on the stack.
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def run(cmd, inp):
    r = subprocess.run(cmd, input=inp, preexec_fn=big_stack, capture_output=True, text=True, timeout=120)
    return r.stdout if r.returncode == 0 else "<exit %d> %s" % (r.returncode, r.stderr[-300:])


def check(name, cases, tmp):
    sub, gen, judge = PORTS[name]
    exe = os.path.join(tmp, name)
    src = os.path.join(ROOT, "corpus", "submissions", sub + ".cpp")
    if name in MAINS:
        with open(exe + ".cpp", "w") as f:
            f.write(open(src).read() + MAINS[name])
        src = exe + ".cpp"
    subprocess.run(["g++", "-std=c++20", "-O1", "-w", "-o", exe, src], check=True)
    port = [sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", os.path.join(HERE, name + ".cp")]
    for i in range(cases):
        inp = gen()
        want, got = run([exe], inp), run(port, inp)
        ok = judge(inp, want, got) if judge else want.split() == got.split()
        if not ok or not want.strip():
            print("%s: MISMATCH on case %d\n--- input\n%s--- original\n%s--- port\n%s" % (name, i, inp, want, got))
            return False
    print("%s: %d cases agree%s" % (name, cases, " (reals compared with a tolerance)" if judge else ""))
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(PORTS)
    cases = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(21)
    with tempfile.TemporaryDirectory() as tmp:
        ok = all([check(n, cases, tmp) for n in names])
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
