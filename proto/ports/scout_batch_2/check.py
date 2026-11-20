#!/usr/bin/env python3
"""Check the batch-2 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs and must print the same output.

    python3 proto/ports/scout_batch_2/check.py            # every port, 60 inputs each
    python3 proto/ports/scout_batch_2/check.py dpv 200    # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))


def tree(n, base=1):
    # a random mix of bushy and path-like trees, so both deep chains and wide fans occur
    near = random.choice([1, 3, n])
    return [(random.randint(max(base, v - near), v - 1), v) for v in range(base + 1, base + n)]


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_apio09p3():
    n = random.randint(1, 7)
    m = random.randint(0, 12)
    roads = [(random.randint(1, n), random.randint(1, n)) for _ in range(m)]
    start = random.randint(1, n)
    reach, todo = {start}, [start]
    while todo:
        u = todo.pop()
        for a, b in roads:
            if a == u and b not in reach:
                reach.add(b), todo.append(b)
    pubs = {random.choice(sorted(reach))} | set(random.sample(range(1, n + 1), random.randint(0, n - 1)))
    return lines((n, m), *roads, *[(random.randint(0, 20),) for _ in range(n)], (start, len(pubs)), sorted(pubs))


def gen_ioi11p4():
    while True:
        n = random.randint(3, 7)
        edges = {}
        for _ in range(random.randint(n, 14)):
            u, v = random.sample(range(n), 2)
            edges[min(u, v), max(u, v)] = random.randint(1, 9)
        exits = random.sample(range(1, n), random.randint(1, n - 2))
        # solvable only if chamber 0 can be forced out: check with the same rule, by fixpoint
        INF = 10 ** 9
        best = [0 if v in exits else INF for v in range(n)]
        for _ in range(n):
            for v in range(n):
                if v in exits:
                    continue
                outs = sorted(best[b if a == v else a] + w for (a, b), w in edges.items() if v in (a, b))
                best[v] = min(INF, outs[1]) if len(outs) > 1 else INF
        if best[0] < INF:
            return lines((n, len(edges), len(exits)), *[(u, v, w) for (u, v), w in edges.items()], exits)


def gen_cco19p5():
    n = random.randint(1, 30)
    pairs = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    sticks = random.sample(pairs, random.randint(0, min(len(pairs), 2 * n)))
    return lines((n, len(sticks)), *sticks)


def gen_cco20p2():
    n = random.randint(1, 12)
    if random.random() < .7:    # mostly feasible: deadlines at or after some valid order
        order = random.sample(range(1, n + 1), n)
        return lines((n,), [random.randint(d, n) for d in order])
    return lines((n,), [random.randint(1, n) for _ in range(n)])


def gen_ccc17s4():
    n = random.randint(1, 9)
    plan = tree(n)
    spare = [random.sample(range(1, n + 1), 2) for _ in range(random.randint(0, 8))] if n > 1 else []
    top = random.choice([2, 4, 9])
    pipes = [(u, v, random.randint(1, top)) for u, v in plan + spare]
    return lines((n, len(pipes), random.randint(0, top + 1)), *pipes)


def gen_noip18p3():
    n = random.randint(2, 25)
    return lines((n, random.randint(1, n - 1)), *[(u, v, random.randint(1, 9)) for u, v in tree(n)])


def gen_dpv():
    n = random.randint(1, 25)
    edges = [e if random.random() < .5 else e[::-1] for e in tree(n)]
    random.shuffle(edges)
    return lines((n, random.choice([2, 3, 7, 10, 1000000007])), *edges)


def gen_ioi08p2():
    while True:
        n = random.randint(2, 25)
        to = [random.choice([u for u in range(1, n + 1) if u != i]) for i in range(1, n + 1)]
        return lines((n,), *[(u, random.randint(1, 9)) for u in to])


def gen_noi15p2():
    n = random.randint(1, 25)
    q = random.randint(1, 25)
    return lines((n,), [random.randint(0, v - 1) for v in range(1, n)], (q,),
                 *[(random.choice(["install", "uninstall"]), random.randint(0, n - 1)) for _ in range(q)])


def gen_ioi14p2():
    n, k = random.randint(1, 20), random.randint(1, 20)
    ops = []
    for _ in range(k):
        l = random.randint(0, n - 1)
        ops.append((random.randint(1, 2), l, random.randint(l, n - 1), random.randint(0, 9)))
    return lines((n, k), *ops)


# ioi14p2's original is a grader function with no `main`; this reads the judge's stdin format and calls it.
MAIN_WALL = """
int main() { int n, k; scanf("%d %d", &n, &k); vector<int> op(k), l(k), r(k), h(k), out(n);
  for (int i = 0; i < k; i++) scanf("%d %d %d %d", &op[i], &l[i], &r[i], &h[i]);
  buildWall(n, k, op.data(), l.data(), r.data(), h.data(), out.data());
  for (int x : out) printf("%d\\n", x); }
"""

PORTS = {  # name: (submission id, input generator, main to append)
    "apio09p3": (5536741, gen_apio09p3, ""),
    "ioi11p4": (3734919, gen_ioi11p4, ""),
    "cco19p5": (3617425, gen_cco19p5, ""),
    "cco20p2": (4561352, gen_cco20p2, ""),
    "ccc17s4": (4899340, gen_ccc17s4, ""),
    "noip18p3": (4002963, gen_noip18p3, ""),
    "dpv": (4133644, gen_dpv, ""),
    "ioi08p2": (5107842, gen_ioi08p2, ""),
    "noi15p2": (3775038, gen_noi15p2, ""),
    "ioi14p2": (4843911, gen_ioi14p2, MAIN_WALL),
}


def check(name, iters, tmp):
    sub, gen, main = PORTS[name]
    src = os.path.join(tmp, name + "_orig.cpp")
    open(src, "w").write(open(os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp")).read() + main)
    exe = os.path.join(tmp, name + "_orig")
    r = subprocess.run(["g++", "-std=c++17", "-O1", "-w", "-o", exe, src], capture_output=True, text=True)
    if r.returncode:
        return "original does not compile: " + r.stderr[:300]
    random.seed(name)
    for i in range(iters):
        data = gen()
        want = subprocess.run([exe], input=data, capture_output=True, text=True).stdout.split()
        got = subprocess.run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", os.path.join(HERE, name + ".cp")],
                             input=data, capture_output=True, text=True)
        if got.returncode or got.stdout.split() != want:
            return f"MISMATCH on input {i}:\n{data}want {' '.join(want)}\ngot  {' '.join(got.stdout.split())} {got.stderr[-400:]}"
    return f"OK ({iters} inputs)"


def main():
    names = [a for a in sys.argv[1:] if a in PORTS] or list(PORTS)
    iters = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            res = check(name, iters, tmp)
            print(f"{name}: {res}", flush=True)
            bad += not res.startswith("OK")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
