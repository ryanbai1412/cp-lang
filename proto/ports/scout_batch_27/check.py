#!/usr/bin/env python3
"""Check the batch-27 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
random inputs, half of them small (a handful of vertices) and half medium
(up to about a hundred). Answers must be equal, except the schedule line of
`noi10p4`, where any valid schedule is accepted.

    python3 proto/ports/scout_batch_27/check.py              # every port, 60 inputs each
    python3 proto/ports/scout_batch_27/check.py ioi18p3 300  # one port, 300 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def size(small, medium):
    """A size for this input: small half of the time, medium otherwise."""
    return R(1, small) if random.random() < 0.5 else R(small, medium)


def tree_edges(n):
    """A random spanning tree on 0..n-1: what `connected` cannot say in a type."""
    return [(R(0, v - 1), v) for v in range(1, n)]


def connected_edges(n, extra):
    edges = set(tree_edges(n))
    for _ in range(extra):
        a, b = R(0, n - 1), R(0, n - 1)
        if a != b and (a, b) not in edges and (b, a) not in edges:
            edges.add((a, b))
    edges = list(edges)
    random.shuffle(edges)
    return [e if R(0, 1) else e[::-1] for e in edges]


def gen_acc7p3():
    n = size(6, 80)
    vals = random.sample(range(1, 4 * n + 20), n)
    m = R(0, n)
    return lines([(n, m), vals] + [(R(1, n), R(1, n)) for _ in range(m)])


def gen_noi15p1():
    rows = [(R(1, 4),)]
    for _ in range(rows[0][0]):
        q, top = size(5, 60), random.choice([3, 8, 30, 10**9])
        rows.append((q,))
        # few inequalities, or almost every test is NO
        rows += [(R(1, top), R(1, top), int(random.random() < 0.8)) for _ in range(q)]
    return lines(rows)


def gen_coci20c6p2():
    n = size(5, 40)
    names = random.sample(["".join(random.choice("abcxyz") for _ in range(R(1, 6))) for _ in range(4 * n)], n)
    names = list(dict.fromkeys(names))
    n = len(names)
    m = R(n, 3 * n)
    # every name must appear on an edge, as in the statement
    ends = [(a, random.choice(names)) for a in names] + [(random.choice(names), random.choice(names)) for _ in range(m - n)]
    q = R(1, 8)
    return lines([(n, m)] + [(a, b, R(1, 20)) for a, b in ends] + [(q,)] +
                 [(random.choice(names), random.choice(names)) for _ in range(q)])


def gen_noi07p1():
    n = size(5, 30)
    edges = connected_edges(n, R(0, 2 * n))
    return lines([(n, len(edges))] + [(a + 1, b + 1, R(1, 4)) for a, b in edges])


def gen_btoi18p1():
    n = size(8, 100)
    if random.random() < 0.9:
        n += n % 2                      # odd n is always -1
    names = ["p%d" % i for i in range(n)]
    love = [random.choice(names) for _ in names]
    if random.random() < 0.3:           # long cycles, which random maps rarely have
        love = names[1:] + names[:1]
        for _ in range(R(0, 3)):
            love[R(0, n - 1)] = random.choice(names)
    return lines([(n,)] + list(zip(names, love)))


def gen_noi10p4():
    # A hidden schedule makes the deadlines and the rules satisfiable: the
    # rules form a DAG and every deadline is at or after the plane's slot.
    n = size(6, 60)
    slot = list(range(1, n + 1))
    random.shuffle(slot)                # slot[i]: when plane i + 1 leaves
    by_slot = sorted(range(n), key=lambda i: slot[i])
    k = [R(s, n) if R(0, 2) else s for s in slot]
    rules = set()
    for _ in range(R(0, 2 * n)):
        a, b = sorted(random.sample(range(n), 2)) if n > 1 else (0, 0)
        if a != b:
            rules.add((by_slot[a] + 1, by_slot[b] + 1))
    return lines([(n, len(rules)), k] + sorted(rules))


def judge_noi10p4(inp, want, got):
    w, g = want.strip().split("\n"), got.strip().split("\n")
    if len(g) != 2 or w[1].split() != g[1].split():
        return False
    rows = [list(map(int, r.split())) for r in inp.strip().split("\n")]
    n, k, rules = rows[0][0], rows[1], rows[2:]
    order = list(map(int, g[0].split()))
    if sorted(order) != list(range(1, n + 1)):
        return False
    at = {p: i + 1 for i, p in enumerate(order)}
    return all(at[p] <= k[p - 1] for p in order) and all(at[a] < at[b] for a, b in rules)


def gen_noi10p3():
    n = size(3, 9)
    top = random.choice([1, 3, 50])
    rows = [(n,)]
    for h, w in ((n + 1, n), (n, n + 1), (n + 1, n), (n, n + 1)):
        rows += [(R(0, top),) for _ in range(h * w)]
    return lines(rows)


def gen_cco19p2():
    n, m = size(5, 14), size(5, 14)
    fill = random.choice([0.15, 0.35, 0.6])
    grid = [["#" if random.random() < fill else "." for _ in range(m)] for _ in range(n)]
    return lines([(n, m)] + [("".join(r),) for r in grid])


def gen_ioi11p1():
    n = size(6, 60)
    edges = connected_edges(n, R(0, n)) if n > 1 else []
    if n == 1:
        n, edges = 2, [(0, 1)]
    if random.random() < 0.3:           # not connected: drop some trails, keep every fountain on one
        keep = edges[: max(1, len(edges) // 2)]
        seen = {v for e in keep for v in e}
        edges = keep + [e for e in edges[len(keep):] if e[0] not in seen or e[1] not in seen]
        seen = {v for e in edges for v in e}
        edges += [(v, R(0, n - 1) if R(0, n - 1) != v else (v + 1) % n) for v in range(n) if v not in seen]
        edges = list(dict.fromkeys(edges))
    q = R(1, 8)
    ks = [random.choice([R(1, 6), R(1, 3 * n), R(1, 10**9)]) for _ in range(q)]
    return lines([(n, len(edges), R(0, n - 1), q)] + edges + [ks])


def gen_ioi18p3():
    n = size(6, 60)
    if n == 1:
        n = 2
    edges = connected_edges(n, random.choice([0, 0, 2, n]))
    q = R(1, 10)
    trips = []
    for _ in range(q):
        # a narrow overlap [l, r], or the answer is nearly always 1
        l = R(0, n - 1)
        r = min(n - 1, l + R(0, 3)) if R(0, 2) else R(l, n - 1)
        trips.append((R(l, n - 1), R(0, r), l, r))
    return lines([(n, len(edges), q)] + edges + trips)


# The IOI originals are grader functions. `ioi11p1` has its own driver
# behind LOCAL, which reads `input.txt`; `ioi18p3` gets this one.
MAIN_IOI18P3 = r"""
int main() {
    int n, m, q;
    scanf("%d %d %d", &n, &m, &q);
    vector<int> x(m), y(m), s(q), e(q), l(q), r(q);
    for (int i = 0; i < m; i++) scanf("%d %d", &x[i], &y[i]);
    for (int i = 0; i < q; i++) scanf("%d %d %d %d", &s[i], &e[i], &l[i], &r[i]);
    for (int a : check_validity(n, x, y, s, e, l, r)) printf("%d\n", a);
}
"""
MAINS = {"ioi18p3": MAIN_IOI18P3}
FLAGS = {"ioi11p1": ["-DLOCAL"]}

PORTS = {
    "coci20c6p2": ("4188022", gen_coci20c6p2, None),
    "noi10p3": ("5672180", gen_noi10p3, None),
    "cco19p2": ("3613676", gen_cco19p2, None),
    "ioi11p1": ("3732440", gen_ioi11p1, None),
    "noi07p1": ("5032737", gen_noi07p1, None),
    "acc7p3": ("3540475", gen_acc7p3, None),
    "noi15p1": ("3775616", gen_noi15p1, None),
    "ioi18p3": ("4727179", gen_ioi18p3, None),
    "btoi18p1": ("4456169", gen_btoi18p1, None),
    "noi10p4": ("5670273", gen_noi10p4, judge_noi10p4),
}


def big_stack():
    # Several originals keep large arrays on the stack.
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def run(cmd, inp, cwd):
    r = subprocess.run(cmd, input=inp, cwd=cwd, preexec_fn=big_stack, capture_output=True, text=True, timeout=120)
    return r.stdout if r.returncode == 0 else "<exit %d> %s" % (r.returncode, r.stderr[-300:])


def check(name, cases, tmp):
    sub, gen, judge = PORTS[name]
    exe = os.path.join(tmp, name)
    src = os.path.join(ROOT, "corpus", "submissions", sub + ".cpp")
    if name in MAINS:
        with open(exe + ".cpp", "w") as f:
            f.write(open(src).read() + MAINS[name])
        src = exe + ".cpp"
    subprocess.run(["g++", "-std=c++20", "-O1", "-w", "-o", exe, src] + FLAGS.get(name, []), check=True)
    port = [sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", os.path.join(HERE, name + ".cp")]
    for i in range(cases):
        inp = gen()
        with open(os.path.join(tmp, "input.txt"), "w") as f:
            f.write(inp)
        want, got = run([exe], inp, tmp), run(port, inp, tmp)
        ok = judge(inp, want, got) if judge else want.split() == got.split()
        if not ok or not want.strip():
            print("%s: MISMATCH on case %d\n--- input\n%s--- original\n%s--- port\n%s" % (name, i, inp, want, got))
            return False
    print("%s: %d cases agree%s" % (name, cases, " (any valid schedule accepted)" if judge else ""))
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(PORTS)
    cases = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(27)
    with tempfile.TemporaryDirectory() as tmp:
        ok = all([check(n, cases, tmp) for n in names])
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
