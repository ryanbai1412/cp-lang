#!/usr/bin/env python3
"""Check the batch-33 ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`); the
original is compiled from `corpus/submissions/<id>.cpp`. Both get the same
random inputs, half small and half medium. Outputs must be equal token by
token unless the port has a judge (several answers are valid, or floats).

    python3 proto/ports/scout_batch_33/check.py              # every port, 40 inputs each
    python3 proto/ports/scout_batch_33/check.py ioi09p7 200  # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def size(small, medium):
    return R(1, small) if random.random() < 0.5 else R(small, medium)


def gen_ioi14p5():
    n, top = size(6, 60), random.choice([3, 1000])
    return lines([(n,), [R(1, top) for _ in range(n)]] + [(R(0, i - 1), R(0, 2)) for i in range(1, n)])


def gen_btoi18p2():
    n, k = size(8, 80), R(1, 5)
    r = R(1, k)
    return lines([(n, k, r), [R(0, k - 1) for _ in range(n)]] + [(b, R(1, 4)) for b in random.sample(range(k), r)])


def gen_btoi15p2():
    # states[i]: the operations active after operation i, oldest first. An undo
    # of level L is only valid when an active operation has a level below L.
    n, ops, states = size(8, 120), [], [[]]
    for i in range(n):
        level = R(1, random.choice([2, 4, 30]))
        hit = [u for u in states[-1] if max(0, -ops[u]) < level]
        if hit and random.random() < 0.6:
            ops.append(-level)
            states.append(states[hit[-1]] + [i])
        else:
            ops.append(R(1, 9))
            states.append(states[-1] + [i])
    return lines([(n,)] + [(o,) for o in ops])


def gen_cco20p4():
    n = 1 + size(6, 30)
    p = random.choice([0.1, 0.5, 0.9])
    return lines([(n,)] + [("".join("R" if random.random() < p else "B" for _ in range(i)),) for i in range(1, n)])


def gen_cco21p2():
    k, m = R(2, 7), random.choice([1, 3, 12])
    pool = list(range(-m, m + 1))
    digits = random.sample(pool, R(1, min(len(pool), k + 2)))
    q, top = size(3, 8), random.choice([20, 2000, 10**15])
    return lines([(k, q, len(digits), m), digits] + [(R(-top, top),) for _ in range(q)])


def gen_ccc20s4():
    n = size(8, 150)
    return "".join(random.choice(random.choice(["ABC", "ABC", "AB", "AC", "BC", "A", "AAAAB"])) for _ in range(n)) + "\n"


def gen_ioi12p6():
    n = 1 + size(6, 60)
    ranks = random.sample(range(n), n)
    rounds, left = [], n
    while left > 1:
        s = R(0, left - 2)
        e = R(s + 1, min(left - 1, s + random.choice([1, 2, 5, left])))
        if random.random() < 0.15:
            s, e = 0, left - 1
        rounds.append((s, e))
        left -= e - s
    return lines([(n, len(rounds), ranks[0]), ranks[1:]] + rounds)


def gen_ioi09p5():
    n, m = size(3, 12), size(5, 40)
    events, inside, order = [], [], random.sample(range(1, m + 1), m)
    while order or inside:
        if order and (not inside or random.random() < 0.6):
            inside.append(order.pop())
            events.append(inside[-1])
        else:
            events.append(-inside.pop(R(0, len(inside) - 1)))
    # a car may only leave once parked: replay and drop departures from the queue
    out, free, queue, parked = [], n, [], set()
    pending = events[:]
    while pending:
        e = pending.pop(0)
        if e < 0 and -e not in parked:
            pending.append(e)
            continue
        out.append(e)
        if e < 0:
            parked.discard(-e)
            free += 1
        else:
            queue.append(e)
        while queue and free:
            parked.add(queue.pop(0))
            free -= 1
    return lines([(n, m)] + [(R(1, 100),) for _ in range(n)] + [(R(1, 1000),) for _ in range(m)] + [(e,) for e in out])


def gen_cco05p6():
    n, q, top = size(4, 30), size(5, 40), random.choice([6, 40])
    wins = []
    for _ in range(n):
        x1, x2 = sorted((R(0, top), R(0, top)))
        y1, y2 = sorted((R(0, top), R(0, top)))
        wins.append((x1, y2, x2, y1))
    return lines([(top, top, n)] + wins + [(q,)] + [(R(0, top), R(0, top)) for _ in range(q)])


def gen_btoi16p1():
    n, p = size(5, 30), random.choice([0.1, 0.3, 0.7])
    order = random.sample(range(1, n + 1), n)       # a hidden tree keeps one root feasible
    ok = {order[i]: {order[R(0, i - 1)]} for i in range(1, n)}
    ok[order[0]] = set()
    rows = []
    for v in range(1, n + 1):
        ok[v] |= {u for u in range(1, n + 1) if u != v and random.random() < p}
        rows.append([len(ok[v])] + sorted(ok[v]))
    return lines([(n,)] + rows)


# name: (submission id, generator, judge or None)
PORTS = {
    "ioi14p5": ("6567598", gen_ioi14p5, None),
    "btoi18p2": ("4456171", gen_btoi18p2, None),
    "btoi15p2": ("5640898", gen_btoi15p2, None),
    "cco20p4": ("4801875", gen_cco20p4, None),
    "cco21p2": ("4543410", gen_cco21p2, None),
    "ccc20s4": ("4324014", gen_ccc20s4, None),
    "ioi12p6": ("4848321", gen_ioi12p6, None),
    "ioi09p5": ("5106592", gen_ioi09p5, None),
    "cco05p6": ("5150823", gen_cco05p6, None),
    "btoi16p1": ("5323961", gen_btoi16p1, None),
}
# a stdin driver appended to an original that is a grader function
MAINS = {
    "ioi12p6": """
int main() {
    int n, c, r;
    scanf("%d %d %d", &n, &c, &r);
    vector<int> k(n), s(c), e(c);
    for (int i = 0; i + 1 < n; i++) scanf("%d", &k[i]);
    for (int i = 0; i < c; i++) scanf("%d %d", &s[i], &e[i]);
    printf("%d\\n", GetBestPosition(n, c, r, k.data(), s.data(), e.data()));
}
""",
}
FLAGS = {}


def big_stack():
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def run(cmd, inp, cwd):
    r = subprocess.run(cmd, input=inp, cwd=cwd, preexec_fn=big_stack, capture_output=True, text=True, timeout=300)
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
        want, got = run([exe], inp, tmp), run(port, inp, tmp)
        ok = judge(inp, want, got) if judge else want.split() == got.split()
        if not ok or not want.strip() or want.startswith("<exit"):
            print("%s: MISMATCH on case %d\n--- input\n%s--- original\n%s--- port\n%s" % (name, i, inp, want, got))
            return False
    print("%s: %d cases agree%s" % (name, cases, " (judged)" if judge else ""))
    return True


def main():
    sys.stdout.reconfigure(line_buffering=True)
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(PORTS)
    cases = next((int(a) for a in sys.argv[1:] if a.isdigit()), 40)
    random.seed(33)
    with tempfile.TemporaryDirectory() as tmp:
        ok = all([check(n, cases, tmp) for n in names])
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
