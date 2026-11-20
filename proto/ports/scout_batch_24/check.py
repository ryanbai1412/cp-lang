#!/usr/bin/env python3
"""Differential test harness for scout_batch_24 statement-first ports.

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable statement-first CP Lang port in
today/ via `proto/cp.py lib-run` on randomized and edge-case inputs.

    python3 proto/ports/scout_batch_24/check.py            # all 10 ports, 20 cases each
    python3 proto/ports/scout_batch_24/check.py noi05p2 30 # single port, 30 cases
"""
import os, random, re, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_ioi07p3(rng, idx):
    if idx == 0:
        return lines((6,), (3, 2), (5, 3), (4, 1), (2, 1), (4, 3), (3, 2))
    if idx == 1:
        return lines((1,), (5, 5))
    n = rng.randint(1, 30)
    max_h = rng.choice([rng.randint(1, 10), rng.randint(10, 60)])
    rows = [(n,)]
    for _ in range(n):
        h = rng.randint(1, max_h)
        k = rng.randint(1, h)
        rows.append((h, k))
    return lines(*rows)


def gen_noi07p4(rng, idx):
    if idx == 0:
        return (
            "5 3\n1 2 3 2 1\n6\nC\nR 2\nP 5 5 2\nCS 4 1\nF\nC\n"
        )
    n = rng.randint(3, 25)
    c = rng.randint(2, 6)
    init = [rng.randint(1, c) for _ in range(n)]
    q = rng.randint(5, 30)
    out = [f"{n} {c}", " ".join(map(str, init)), str(q)]
    for _ in range(q):
        op = rng.choice(["R", "F", "S", "P", "C", "CS"])
        if op == "R":
            out.append(f"R {rng.randint(0, n - 1)}")
        elif op == "F":
            out.append("F")
        elif op == "S":
            out.append(f"S {rng.randint(1, n)} {rng.randint(1, n)}")
        elif op == "P":
            out.append(f"P {rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, c)}")
        elif op == "C":
            out.append("C")
        else:
            out.append(f"CS {rng.randint(1, n)} {rng.randint(1, n)}")
    return "\n".join(out) + "\n"


def gen_noip22p4(rng, idx):
    if idx == 0:
        return lines((0, 2), [2, 1], [1, 2], (1,), (1, 2))
    n = rng.randint(1, 35)
    a = list(range(1, n + 1))
    b = list(range(1, n + 1))
    rng.shuffle(a)
    rng.shuffle(b)
    q = rng.randint(1, 40)
    qs = []
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        qs.append((l, r))
    return lines((0, n), a, b, (q,), *qs)


def gen_noi20p2(rng, idx):
    if idx == 0:
        return lines((5,), (1, 2), (2, 3), (3, 4), (3, 5), (2,), (1, 3), (2, 5))
    if idx == 1:
        return lines((1,), (0,))
    n = rng.randint(2, 30)
    parent = [0] * (n + 1)
    edges = []
    for u in range(2, n + 1):
        p = rng.randint(1, u - 1)
        parent[u] = p
        edges.append((p, u))
    # Build ancestor-descendant constraints (u strict ancestor of v).
    anc_pairs = []
    for v in range(2, n + 1):
        cur = parent[v]
        while cur != 0:
            anc_pairs.append((cur, v))
            cur = parent[cur]
    m = rng.randint(0, min(20, len(anc_pairs)))
    paths = [rng.choice(anc_pairs) for _ in range(m)] if m > 0 else []
    return lines((n,), *edges, (m,), *paths)


def gen_noi05p2(rng, idx):
    if idx == 0:
        return (
            "9 8\n2 -6 3 5 1 -5 -3 6 3\n"
            "GET-SUM 5 4\nMAX-SUM\nINSERT 8 3 -5 7 2\nDELETE 12 1\n"
            "MAKE-SAME 3 3 2\nREVERSE 3 6\nGET-SUM 5 4\nMAX-SUM\n"
        )
    n = rng.randint(1, 15)
    m = rng.randint(5, 30)
    cur_len = n
    init = [rng.randint(-50, 50) for _ in range(n)]
    out = [f"{n} {m}", " ".join(map(str, init))]
    for _ in range(m):
        choices = ["INSERT", "MAKE-SAME", "REVERSE", "GET-SUM", "MAX-SUM"]
        if cur_len > 1:
            choices.append("DELETE")
        op = rng.choice(choices)
        if op == "INSERT":
            posi = rng.randint(0, cur_len)
            tot = rng.randint(1, 6)
            vals = [rng.randint(-50, 50) for _ in range(tot)]
            out.append("INSERT {} {} {}".format(posi, tot, " ".join(map(str, vals))).strip())
            cur_len += tot
        elif op == "DELETE":
            tot = rng.randint(1, cur_len - 1)
            posi = rng.randint(1, cur_len - tot + 1)
            out.append(f"DELETE {posi} {tot}")
            cur_len -= tot
        elif op == "MAKE-SAME":
            tot = rng.randint(1, cur_len)
            posi = rng.randint(1, cur_len - tot + 1)
            c = rng.randint(-50, 50)
            out.append(f"MAKE-SAME {posi} {tot} {c}")
        elif op == "REVERSE":
            tot = rng.randint(1, cur_len)
            posi = rng.randint(1, cur_len - tot + 1)
            out.append(f"REVERSE {posi} {tot}")
        elif op == "GET-SUM":
            tot = rng.randint(0, cur_len)
            posi = rng.randint(1, max(1, cur_len - tot + 1)) if tot > 0 else 1
            out.append(f"GET-SUM {posi} {tot}")
        else:
            out.append("MAX-SUM")
    return "\n".join(out) + "\n"


def gen_cco18p5(rng, idx):
    if idx == 0:
        return lines((5, 3, 4), [1, 5, 2, 4, 3], (2, 1), (4, 10), (3, 10), (1, 20))
    n = rng.randint(2, 40)
    k = rng.randint(2, n)
    q = rng.randint(0, 40)
    arr = [rng.randint(1, 50) for _ in range(n)]
    upds = [(rng.randint(1, n), rng.randint(1, 50)) for _ in range(q)]
    return lines((n, k, q), arr, *upds)


def gen_ioi13p3io(rng, idx):
    if idx == 0:
        return (
            "3 3\n"
            "0 2\n0 3\n1 2\n"
            "1 4 1\n0 4 0\n"
            "4\n3 0 2\n1 0 1 1\n2 0 0 10\n3 0 2\n"
        )
    r = rng.randint(2, 20)
    c = rng.randint(2, 10)
    rows = [(r, c)]
    for _ in range(r):
        rows.append([rng.randint(0, 30) for _ in range(c - 1)])
    for _ in range(r - 1):
        rows.append([rng.randint(0, 30) for _ in range(c)])
    e = rng.randint(4, 25)
    rows.append((e,))
    for _ in range(e):
        ev = rng.choice([1, 2, 3, 3])
        if ev == 1:
            rows.append((1, rng.randint(0, r - 1), rng.randint(0, c - 2), rng.randint(0, 30)))
        elif ev == 2:
            rows.append((2, rng.randint(0, r - 2), rng.randint(0, c - 1), rng.randint(0, 30)))
        else:
            rows.append((3, rng.randint(0, c - 1), rng.randint(0, c - 1)))
    return lines(*rows)


def gen_dmpg15g6(rng, idx):
    if idx == 0:
        return lines((5, 3), [1, 2, 1, 3, 2], (4,), (1, 5), (1, 3), (2, 4), (4, 5))
    n = rng.randint(1, 50)
    m = rng.randint(1, 10)
    # Bias flavors so both YES and NO cases appear frequently.
    pool = [rng.randint(1, m) for _ in range(min(m, 4))]
    arr = [rng.choice(pool) if rng.random() < 0.7 else rng.randint(1, m) for _ in range(n)]
    q = rng.randint(1, 40)
    qs = []
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        qs.append((l, r))
    return lines((n, m), arr, (q,), *qs)


def gen_ceoi22p1(rng, idx):
    if idx == 0:
        return lines((4, 4), [3, 1, 4, 2], (0, 2), (1, 1), (1, 3), (5, 4))
    n = 2 * rng.randint(1, 20)
    deck = list(range(1, n + 1))
    rng.shuffle(deck)
    q = rng.randint(1, 35)
    qs = [(rng.choice([0, 1, 2, rng.randint(0, n + 5), 10**9]), rng.randint(1, n)) for _ in range(q)]
    return lines((n, q), deck, *qs)


def gen_cco16p5(rng, idx):
    if idx == 0:
        return lines((4, 4), (2,), (2, 2), (3, 4), (1,))
    if idx == 1:
        return lines((10, 10), (3,), (1, 1), (5, 5), (10, 10), (0,))
    n = rng.choice([rng.randint(1, 30), rng.randint(100, 10000)])
    m = rng.choice([rng.randint(1, 30), rng.randint(100, 10000)])
    k = rng.randint(1, min(25, n * m))
    pts = set()
    while len(pts) < k:
        pts.add((rng.randint(1, n), rng.randint(1, m)))
    q = rng.choice([0, 1, rng.randint(1, max(n, m) + 5)])
    return lines((n, m), (k,), *sorted(pts), (q,))


GENS = {
    "ioi07p3": gen_ioi07p3,
    "noi07p4": gen_noi07p4,
    "noip22p4": gen_noip22p4,
    "noi20p2": gen_noi20p2,
    "noi05p2": gen_noi05p2,
    "cco18p5": gen_cco18p5,
    "ioi13p3io": gen_ioi13p3io,
    "dmpg15g6": gen_dmpg15g6,
    "ceoi22p1": gen_ceoi22p1,
    "cco16p5": gen_cco16p5,
}


def outputs_match(name, ours_out, ref_out):
    return ours_out.split() == ref_out.split()


def check_one(name, iters, tmp):
    port = os.path.join(HERE, "today", name + ".cp")
    src = open(port).read()
    m = re.search(r"^// original: (\S+)", src, re.M)
    orig = os.path.join(ROOT, m.group(1))
    exe = os.path.join(tmp, name + "_ref")
    comp = subprocess.run(
        ["g++", "-std=c++17", "-O2", "-o", exe, orig],
        capture_output=True, text=True, cwd=ROOT,
    )
    if comp.returncode != 0:
        return False, f"g++ failed: {comp.stderr.splitlines()[:4]}"

    # Compile CP Lang port once via lib-run on case 0, or run directly per case.
    def _unlimited_stack():
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))

    rng = random.Random(20261124 + sum(ord(c) for c in name) * 31)
    gen = GENS[name]
    for i in range(iters):
        inp = gen(rng, i)
        ref = subprocess.run([exe], input=inp, capture_output=True, text=True, timeout=10, preexec_fn=_unlimited_stack)
        if ref.returncode != 0:
            return False, f"ref crashed on case {i}: {ref.stderr}"
        ours = subprocess.run(
            ["python3", CP, "lib-run", port],
            input=inp, capture_output=True, text=True, cwd=ROOT, timeout=30,
        )
        if ours.returncode != 0:
            return False, f"lib-run failed on case {i}:\n{ours.stderr.strip()}\ninput:\n{inp}"
        if not outputs_match(name, ours.stdout, ref.stdout):
            return False, (
                f"MISMATCH on case {i}:\ninput:\n{inp}"
                f"ours: {ours.stdout.strip()}\nref:  {ref.stdout.strip()}"
            )
    return True, f"OK ({iters}/{iters} cases match {os.path.basename(orig)})"


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(GENS.keys())
    iters = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    failed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            ok, msg = check_one(name, iters, tmp)
            status = "PASS" if ok else "FAIL"
            print(f"[{status}] {name:10s} {msg}", flush=True)
            if not ok:
                failed += 1
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
