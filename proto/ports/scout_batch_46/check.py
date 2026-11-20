#!/usr/bin/env python3
"""Differential test harness for scout_batch_46 (AAC, YAC, SAC classics, part 2).

Compiles each original AC C++ submission from corpus/submissions/ and each
runnable port in today/ (via `proto/cp.py build` on a temporary copy so tracked
sources are never stamped, then `g++ -std=c++17 -O2 -fsanitize=address,undefined`)
and requires identical output on small and medium random inputs. yac1p3 is
interactive: the original plays a seeded random opponent through a Python
judge that checks every move is legal, and the port must reproduce that game
(see check_yac1p3 for why the port is replayed instead of played live).

    python3 proto/ports/scout_batch_46/check.py               # all 10 ports, 150 cases
    python3 proto/ports/scout_batch_46/check.py yac6p5 300    # one port, 300 cases
"""
import os, random, resource, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")
SEED = 46


def pick_size(rng, idx, small, medium):
    return rng.randint(*(small if idx % 3 else medium))


def gen_aac4p4(rng, idx):
    n = pick_size(rng, idx, (1, 5), (6, 50))
    m = pick_size(rng, idx, (1, 5), (6, 50))
    s, f = rng.randint(1, n), rng.randint(1, n)
    if rng.random() < 0.3:
        f = s
    kind = rng.randrange(4)
    if kind == 0:
        l, r = 0, rng.randint(0, n * m)
    elif kind == 1:
        l = rng.randint(0, n)
        r = rng.randint(l, n * m)
    else:
        l, r = sorted([rng.randint(0, n * m), rng.randint(0, n * m)])
    pool = list(range(1, n + 1))
    if rng.random() < 0.5 and n > 1:
        pool.remove(f)
    arr = [rng.choice(pool) for _ in range(m)]
    return f"{n} {m} {s} {f} {l} {r}\n" + " ".join(map(str, arr)) + "\n"


def gen_aac5p4(rng, idx):
    n = pick_size(rng, idx, (1, 30), (31, 3000))
    x = rng.choice([rng.randint(0, 10), rng.randint(0, 10**9)])
    return f"{n} {x}\n"


def random_tree(rng, n):
    shape = rng.randrange(4)
    edges = []
    for i in range(2, n + 1):
        if shape == 0:
            p = i - 1
        elif shape == 1:
            p = 1
        elif shape == 2:
            p = rng.randint(max(1, i - 3), i - 1)
        else:
            p = rng.randint(1, i - 1)
        edges.append((p, i) if rng.random() < 0.5 else (i, p))
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    rng.shuffle(edges)
    return [(perm[u - 1], perm[v - 1]) for u, v in edges]


def gen_yac1p2(rng, idx):
    n = pick_size(rng, idx, (1, 12), (30, 400))
    bias = rng.choice([0.1, 0.5, 0.9])
    s = "".join("B" if rng.random() < bias else "W" for _ in range(n))
    return f"{n}\n{s}\n" + "".join(f"{u} {v}\n" for u, v in random_tree(rng, n))


def gen_yac6p4(rng, idx):
    n = pick_size(rng, idx, (1, 6), (10, 300))
    m = rng.randint(1, 100)
    hi = rng.choice([5, 30, 100])
    a = [rng.randint(1, hi) for _ in range(n)]
    return f"{n} {m}\n" + " ".join(map(str, a)) + "\n"


def gen_yac6p5(rng, idx):
    n = pick_size(rng, idx, (1, 10), (30, 400))
    span = rng.choice([3, 50, 10**9])
    kinds = rng.choice([[0, 1, 2, 3], [0], [2], [0, 2], [1, 3]])
    return f"{n}\n" + "".join(
        f"{rng.randint(-span, span)} {rng.randint(-span, span)} {rng.choice(kinds)}\n" for _ in range(n))


def gen_yac7p2(rng, idx):
    t = pick_size(rng, idx, (1, 6), (3, 10))
    hi = 9 if idx % 3 else 40
    return f"{t}\n" + "".join(f"{rng.randint(1, hi)} {rng.randint(1, hi)}\n" for _ in range(t))


def gen_sac22cc2p3(rng, idx):
    k = pick_size(rng, idx, (1, 4), (5, 9))
    n = 1 << k
    # n - 1 forks over 2n - 1 nodes: grow a random full binary tree from node 1
    leaves, nxt, lines = [1], 2, []
    deep = rng.random() < 0.3
    for _ in range(n - 1):
        u = leaves.pop() if deep else leaves.pop(rng.randrange(len(leaves)))
        a, b = nxt, nxt + 1
        nxt += 2
        if rng.random() < 0.5:
            a, b = b, a
        leaves += [a, b]
        w = rng.randint(-20, 20) if rng.random() < 0.5 else rng.randint(-10**9, 10**9)
        lines.append(f"{u} {a} {b} {w}")
    rng.shuffle(lines)
    return f"{k} {rng.randint(0, 10**9)}\n" + "\n".join(lines) + "\n"


def gen_sac22cc2p5(rng, idx):
    n = pick_size(rng, idx, (1, 10), (30, 400))
    q = pick_size(rng, idx, (1, 15), (30, 300))
    hi = rng.choice([5, 10**9])
    lines = [f"{n} {q}", " ".join(str(rng.randint(1, hi)) for _ in range(n))]
    for _ in range(q):
        if rng.random() < 0.4:
            lines.append(f"1 {rng.randint(1, n)} {rng.randint(1, hi)}")
        else:
            l, r = sorted([rng.randint(1, n), rng.randint(1, n)])
            lines.append(f"2 {l} {r}")
    return "\n".join(lines) + "\n"


def gen_sac22cc3p3(rng, idx):
    n = pick_size(rng, idx, (1, 8), (20, 200))
    digits = rng.choice([1, 2, 4, 9])
    a = [rng.randint(0, 10 ** rng.randint(1, digits)) for _ in range(n)]
    return f"{n}\n" + " ".join(map(str, a)) + "\n"


def gen_yac1p3(rng, idx):
    n = pick_size(rng, idx, (1, 6), (7, 14))
    return (n, rng.randint(1, 3), rng.getrandbits(32))


def play_yac1p3(cmd, case):
    """Judge one live program. Returns (judge lines, program lines), or None on an illegal move."""
    n, t, seed = case
    rng = random.Random(seed)
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    said, heard = [], []

    def say(line):
        said.append(line)
        p.stdin.write(line + "\n")
        p.stdin.flush()

    def hear():
        line = p.stdout.readline().split()
        heard.append(line)
        return list(map(int, line))

    try:
        say(f"{n} {t}")
        for _ in range(t):
            used = [[False] * (n + 2) for _ in range(n + 2)]

            def claim(x, y):
                if not (1 <= x <= n and 1 <= y <= n) or used[x][y]:
                    return False
                for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
                    used[x + dx][y + dy] = True
                return True

            first = hear()
            if len(first) != 1 or first[0] not in (1, 2):
                return None
            if first[0] == 1:
                mv = hear()
                if len(mv) != 2 or not claim(*mv):
                    return None
            while True:
                free = [(x, y) for x in range(1, n + 1) for y in range(1, n + 1) if not used[x][y]]
                if not free:
                    say("0 0")
                    break
                x, y = rng.choice(free)
                claim(x, y)
                say(f"{x} {y}")
                mv = hear()
                # the program must always have a reply: it is the one who wins
                if len(mv) != 2 or not claim(*mv):
                    return None
        p.stdin.close()
        if p.stdout.read().strip() or p.wait(timeout=10) != 0:
            return None
        return said, heard
    finally:
        p.kill()


def check_yac1p3(ref, port, case):
    """The original plays the judge live. `cp.py build` output only flushes at
    exit, so the port is replayed: it gets the judge's side of that game on
    stdin and must print the original's side move for move. The judge's moves
    depend only on the program's, so an equal transcript is the same legal game."""
    game = play_yac1p3(ref, case)
    if game is None:
        return f"original made an illegal move on {case}"
    said, heard = game
    got = sh(port, input="\n".join(said) + "\n")
    if got.returncode != 0 or [l.split() for l in got.stdout.strip().splitlines()] != heard:
        return f"case {case}\n--- want\n{heard}\n--- got\n{got.stdout[:300]}\n{got.stderr[:400]}"
    return None


PORTS = {
    "aac4p4": (4140376, gen_aac4p4),
    "aac5p4": (4239900, gen_aac5p4),
    "yac1p2": (4376253, gen_yac1p2),
    "yac1p3": (4380584, gen_yac1p3),
    "yac6p4": (5149666, gen_yac6p4),
    "yac6p5": (5149823, gen_yac6p5),
    "yac7p2": (5705152, gen_yac7p2),
    "sac22cc2p3": (4121049, gen_sac22cc2p3),
    "sac22cc2p5": (4121099, gen_sac22cc2p5),
    "sac22cc3p3": (4331598, gen_sac22cc3p3),
}
INTERACTIVE = {"yac1p3": check_yac1p3}


def raise_stack():
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    resource.setrlimit(resource.RLIMIT_STACK, (hard, hard))


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def build(name, sub, tmp):
    ref = os.path.join(tmp, name + ".ref")
    r = sh(["g++", "-std=c++17", "-O2", "-w", "-o", ref,
            os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp")])
    assert r.returncode == 0, r.stderr
    src = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), src)
    r = sh([sys.executable, CP, "build", src], cwd=os.path.join(ROOT, "proto"))
    assert r.returncode == 0 and os.path.exists(src + "p"), r.stdout + r.stderr
    port = os.path.join(tmp, name + ".port")
    r = sh(["g++", "-std=c++17", "-O2", "-fsanitize=address,undefined",
            "-fno-omit-frame-pointer", "-w", "-o", port, src + "p"])
    assert r.returncode == 0, r.stderr
    return [ref], [port]


def run_case(name, ref, port, case):
    """Returns None when the port agrees with the original, else a failure report."""
    if name in INTERACTIVE:
        return INTERACTIVE[name](ref, port, case)
    want = sh(ref, input=case, preexec_fn=raise_stack)
    got = sh(port, input=case)
    same = [l.split() for l in want.stdout.strip().splitlines()] == [
        l.split() for l in got.stdout.strip().splitlines()]
    if want.returncode != 0 or got.returncode != 0 or not same:
        return (f"exit {want.returncode} vs {got.returncode}\n{case[:600]}\n--- want\n"
                f"{want.stdout[:300]}\n--- got\n{got.stdout[:300]}\n{got.stderr[:400]}")
    return None


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(PORTS)
    cases = int(sys.argv[2]) if len(sys.argv) > 2 else 150
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            sub, gen = PORTS[name]
            ref, port = build(name, sub, tmp)
            rng = random.Random(SEED)
            for idx in range(cases):
                report = run_case(name, ref, port, gen(rng, idx))
                if report:
                    bad += 1
                    print(f"FAIL {name} case {idx}: {report}")
                    break
            else:
                print(f"ok   {name}: {cases} cases match corpus/submissions/{sub}.cpp (ASan+UBSan clean)")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
