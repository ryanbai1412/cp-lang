#!/usr/bin/env python3
"""Check the batch-26 scout ports (hard trees) against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random trees and their outputs must be equal. Three ports need more:

- ioi18p3: the original is the IOI grader interface (`check_validity`), so a
  stdin/stdout `main` in the DMOJ format is appended before compiling.
- btoi15p3: any optimal answer is accepted. The port must print as many
  links as the original, and the tree plus its links must have no bridge.
- ceoi22p3: interactive. A judge with hidden edge lengths is played here in
  Python and both programs are scored against the true distances. The
  interpreter reads all of stdin before it starts, so the port is run
  twice: once to collect its questions (they depend only on the trees),
  then again with the replies appended.

    python3 proto/ports/scout_batch_26/check.py              # every port, 60 inputs each
    python3 proto/ports/scout_batch_26/check.py noi20p2 300  # one port, 300 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def parents(n, spread=4):
    """par[v] for v in 2..n, each below a recent earlier vertex: `spread` 1 is a
    path, a large spread is bushy."""
    return {v: R(max(1, v - R(1, spread)), v - 1) for v in range(2, n + 1)}


def tree_edges(n, spread=4):
    return [(p, v) if R(0, 1) else (v, p) for v, p in parents(n, spread).items()]


def gen_noip19p6():
    rows = [[R(1, 3)]]
    for _ in range(rows[0][0]):
        n = R(2, 13)
        rows += [[n]] + tree_edges(n, R(1, 6))
    return lines(rows)


def gen_ioi09p7():
    n, r, q = R(1, 14), R(2, 4), R(1, 8)
    rows = [[n, r, q], [R(1, r)]] + [[p, R(1, r)] for p in parents(n).values()]
    for _ in range(q):
        a = R(1, r)
        rows.append([a, random.choice([b for b in range(1, r + 1) if b != a])])
    return lines(rows)


def gen_dmopc20c7p4():
    n, q = R(1, 10), R(1, 12)
    rows = [[n, q]] + tree_edges(n)
    for i in range(q):
        # the last one is a question, so every case prints something
        rows.append([1, R(1, n)] if i < q - 1 and R(0, 1) else [2, R(1, n), R(1, n)])
    return lines(rows)


def gen_mmcc14p3():
    n = R(1, 14)
    return lines([[n, R(1, n)]] + tree_edges(n, R(1, 4)))


def gen_btoi15p3():
    n = R(3, 14)
    return lines([[n]] + tree_edges(n, R(1, 5)))


def gen_noip19p2():
    n = R(1, 14)
    return lines([[n], ["".join(random.choice("()") for _ in range(n))], list(parents(n, 3).values())])


def gen_sdoi17r1p2():
    n, q = R(1, 12), R(1, 14)
    rows = [[n, q]] + tree_edges(n)
    for i in range(q):
        kind = R(1, 3) if i < q - 1 else R(2, 3)
        rows.append([2, R(1, n), R(1, n)] if kind == 2 else [kind, R(1, n)])
    return lines(rows)


def gen_noi20p2():
    n = R(2, 12)
    par = parents(n, 3)
    rows = [[n]] + [(p, v) if R(0, 1) else (v, p) for v, p in par.items()]
    m = R(1, 6)
    rows.append([m])
    for _ in range(m):
        v = R(2, n)
        u = par[v]
        while u != 1 and R(0, 1):
            u = par[u]
        rows.append([u, v])
    return lines(rows)


def gen_ioi18p3():
    n = R(2, 10)
    roads = {(R(0, i - 1), i) for i in range(1, n)}
    for _ in range(R(0, 4)):
        a, b = R(0, n - 1), R(0, n - 1)
        if a != b:
            roads.add((min(a, b), max(a, b)))
    roads = list(roads)
    random.shuffle(roads)
    q = R(1, 8)
    rows = [[n, len(roads), q]] + roads
    for _ in range(q):
        l = R(0, n - 1)
        r = R(l, n - 1)
        rows.append([R(l, n - 1), R(0, r), l, r])
    return lines(rows)


def bridgeless(inp, want, got):
    """btoi15p3: as many links as the original, and no bridge once they are added."""
    a, b = want.split(), got.split()
    if not b or a[0] != b[0] or len(b) != 1 + 2 * int(b[0]):
        return False
    tok = list(map(int, inp.split()))
    n = tok[0]
    edges = list(zip(tok[1::2], tok[2::2])) + list(zip(map(int, b[1::2]), map(int, b[2::2])))

    def connected(skip):
        adj = {v: [] for v in range(1, n + 1)}
        for i, (u, v) in enumerate(edges):
            if i != skip:
                adj[u].append(v)
                adj[v].append(u)
        seen, todo = {1}, [1]
        while todo:
            for y in adj[todo.pop()]:
                if y not in seen:
                    seen.add(y)
                    todo.append(y)
        return len(seen) == n

    return all(connected(i) for i in range(len(edges)))


MAINS = {
    "ioi18p3": """
int main() {
    int n, m, q;
    scanf("%d %d %d", &n, &m, &q);
    vi us(m), vs(m), s(q), e(q), l(q), r(q);
    for (int i = 0; i < m; i++) scanf("%d %d", &us[i], &vs[i]);
    for (int i = 0; i < q; i++) scanf("%d %d %d %d", &s[i], &e[i], &l[i], &r[i]);
    for (int x : check_validity(n, us, vs, s, e, l, r)) printf("%d\\n", x);
}
""",
}

PORTS = {
    "noip19p6": ("5148136", gen_noip19p6, None),
    "ioi09p7": ("5032754", gen_ioi09p7, None),
    "dmopc20c7p4": ("3686586", gen_dmopc20c7p4, None),
    "mmcc14p3": ("5120083", gen_mmcc14p3, None),
    "btoi15p3": ("5640865", gen_btoi15p3, bridgeless),
    "noip19p2": ("5127828", gen_noip19p2, None),
    "sdoi17r1p2": ("5463990", gen_sdoi17r1p2, None),
    "noi20p2": ("5655455", gen_noi20p2, None),
    "ioi18p3": ("4727179", gen_ioi18p3, None),
    "ceoi22p3": ("4728198", None, None),
}


def big_stack():
    # Several originals keep large arrays on the stack.
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def run(cmd, inp, partial=False):
    r = subprocess.run(cmd, input=inp, preexec_fn=big_stack, capture_output=True, text=True, timeout=120)
    return r.stdout if r.returncode == 0 or partial else "<exit %d> %s" % (r.returncode, r.stderr[-300:])


def prize_case():
    """ceoi22p3: two random trees with hidden lengths. Returns the first part
    of the input and a judge that turns a program's questions into the rest
    of the input and the expected answers."""
    n = R(2, 12)
    k, t = R(2, n), R(1, 6)

    def tree():
        order = random.sample(range(1, n + 1), n)
        par, w = {order[0]: -1}, {order[0]: 0}
        for i in range(1, n):
            par[order[i]], w[order[i]] = order[R(max(0, i - 3), i - 1)], R(1, 9)
        return par, w

    def ups(par, v):
        out = []
        while v != -1:
            out.append(v)
            v = par[v]
        return out

    def lca(par, a, b):
        above = set(ups(par, a))
        return next(x for x in ups(par, b) if x in above)

    def depth(par, w, v):
        return sum(w[x] for x in ups(par, v))

    trees = [tree(), tree()]
    head = lines([[n, k, k - 1, t]] + [[par[v] for v in range(1, n + 1)] for par, _ in trees])

    def judge(asked):
        tok = asked.split()
        picked, rest = list(map(int, tok[:k])), tok[k:]
        if len(set(picked)) != k or "!" not in rest:
            return None
        rest = rest[:rest.index("!")]
        if len(rest) % 3 or len(rest) // 3 > k - 1:
            return None
        rows = []
        for i in range(0, len(rest), 3):
            a, b = int(rest[i + 1]), int(rest[i + 2])
            if rest[i] != "?" or a not in picked or b not in picked:
                return None
            row = []
            for par, w in trees:
                c = depth(par, w, lca(par, a, b))
                row += [depth(par, w, a) - c, depth(par, w, b) - c]
            rows.append(row)
        pairs = [(random.choice(picked), random.choice(picked)) for _ in range(t)]
        want = [[depth(par, w, a) + depth(par, w, b) - 2 * depth(par, w, lca(par, a, b)) for par, w in trees] for a, b in pairs]
        return lines(rows + pairs), lines(want), len(tok[:k]) + len(rest) + 1

    return head, judge


def check_prize(exe, port, cases):
    for i in range(cases):
        head, judge = prize_case()
        for who, cmd in (("original", [exe]), ("port", port)):
            asked = run(cmd, head, partial=True)
            verdict = judge(asked)
            if verdict is None:
                print("ceoi22p3: %s asked something invalid on case %d\n--- input\n%s--- asked\n%s" % (who, i, head, asked))
                return False
            tail, want, skip = verdict
            got = run(cmd, head + tail)
            if got.split()[skip:] != want.split():
                print("ceoi22p3: %s MISMATCH on case %d\n--- input\n%s--- want\n%s--- got\n%s" % (who, i, head + tail, want, got))
                return False
    print("ceoi22p3: %d cases agree with the true distances (interactive, original and port)" % cases)
    return True


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
    if gen is None:
        return check_prize(exe, port, cases)
    for i in range(cases):
        inp = gen()
        want, got = run([exe], inp), run(port, inp)
        ok = judge(inp, want, got) if judge else want.split() == got.split()
        if not ok or not want.strip():
            print("%s: MISMATCH on case %d\n--- input\n%s--- original\n%s--- port\n%s" % (name, i, inp, want, got))
            return False
    print("%s: %d cases agree%s" % (name, cases, " (any optimal answer accepted)" if judge else ""))
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(PORTS)
    cases = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(26)
    with tempfile.TemporaryDirectory() as tmp:
        ok = all([check(n, cases, tmp) for n in names])
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
