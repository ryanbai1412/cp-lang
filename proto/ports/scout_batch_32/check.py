#!/usr/bin/env python3
"""Differential test harness for scout_batch_32 statement-first ports.

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable CP Lang port in today/ via
`proto/cp.py lib-run` on edge-case and randomized small/medium inputs.

    python3 proto/ports/scout_batch_32/check.py              # all 10 ports, 40 cases each
    python3 proto/ports/scout_batch_32/check.py ioi04p2hard 200  # single port, 200 cases
"""
import os, random, re, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")
L = 10**9 + 1


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def size(rng, small, medium):
    """Mostly small sizes, every fourth case medium."""
    return rng.randint(1, medium if rng.randrange(4) == 0 else small)


def tree_edges(rng, n):
    shape = rng.randrange(3)
    edges = []
    for v in range(2, n + 1):
        p = v - 1 if shape == 0 else 1 if shape == 1 else rng.randint(1, v - 1)
        edges.append((p, v) if rng.randrange(2) else (v, p))
    rng.shuffle(edges)
    return edges


def gen_iati21a22(rng, idx):
    if idx == 0:
        return lines((1,), (0, 0))
    n = size(rng, 8, 60)
    c = rng.choice([3, 10, 10**9])
    return lines((n,), *[(rng.randint(-c, c), rng.randint(-c, c)) for _ in range(n)])


def gen_aac5p3(rng, idx):
    # Distinct edge weights: equal weights at one vertex are split by an
    # unstable std::sort in the reference.
    n, m = size(rng, 8, 50), size(rng, 8, 60)
    c = rng.choice([40, 300, 10**9])
    ws = rng.sample(range(1, max(c, n) + 1), n - 1)
    edges = [(u, v, w) for (u, v), w in zip(tree_edges(rng, n), ws)]
    return lines((n, m), *edges, [rng.randint(1, c) for _ in range(m)])


def gen_ucrpc21e(rng, idx):
    t = rng.choice([8, 50, 10**6])
    n, m = size(rng, 5, 40), size(rng, 5, 40)
    rows = [(t, n, m)]
    for _ in range(n + m):
        l = rng.randint(1, t)
        rows.append((l, rng.randint(l, t), rng.randrange(2)))
    return lines(*rows)


def gen_noimps23p1(rng, idx):
    n = size(rng, 8, 60)
    m = size(rng, 8, 60)
    rows = [(n, m, rng.randint(1, n))]
    for _ in range(m):
        l = rng.randint(1, n)
        rows.append((l, rng.randint(l, n)))
    return lines(*rows)


def gen_noip10p1(rng, idx):
    n = size(rng, 10, 200)
    m = rng.randint(1, rng.choice([2, 5, 100]))
    c = rng.choice([3, 8, 1000])
    return lines((m, n), [rng.randint(0, c) for _ in range(n)])


def gen_dmopc20c4p4(rng, idx):
    # a[i] may not exceed what the reference can still reach at step i.
    n = size(rng, 10, 150)
    have, spare, arr = 0, 0, []
    for _ in range(n):
        a = rng.randint(0, have + spare) if rng.randrange(3) else min(have, have + spare)
        take = max(0, a - have)
        have, spare = have + take, spare - take
        if a == have:
            have += 1
        else:
            spare += 1
        arr.append(a)
    return lines((n,), arr)


def gen_996warmup8p3(rng, idx):
    n = size(rng, 5, 40)
    c = rng.choice([40, 1000, L - 1])
    pos = sorted(rng.sample(range(1, c + 1), n))
    k = rng.choice([3, 40, 10**6])
    return lines((n, rng.randint(1, k), rng.randint(1, k)), pos, [rng.randrange(2) for _ in range(n)])


def gen_ioi04p2hard(rng, idx):
    if idx == 0:
        return lines((1,), (3, 4))
    n = size(rng, 6, 60)
    c = rng.choice([4, 50, 10**9])
    return lines((n,), *[(rng.randint(0, c), rng.randint(0, c)) for _ in range(n)])


def gen_rpeng3(rng, idx):
    n = size(rng, 6, 60)
    top = rng.choice([6, 40, 400])
    days = rng.choice([3, 20, 5000])
    u, d = rng.randint(1, 20), rng.randint(1, 20)
    rows = [(n, u, d, rng.randint(1, top))]
    for _ in range(n):
        rows.append((rng.randint(1, days), rng.randint(1, top), rng.randint(1, rng.choice([10, 4000]))))
    return lines(*rows)


def gen_ceoi22p5(rng, idx):
    n, m = size(rng, 5, 40), size(rng, 5, 40)
    c = rng.choice([5, 60, 10**9])
    return lines((n, m, rng.randint(1, c)), [rng.randint(0, c) for _ in range(n + m)])


GENS = {
    "ioi04p2hard": gen_ioi04p2hard,
    "rpeng3": gen_rpeng3,
    "ceoi22p5": gen_ceoi22p5,
    "iati21a22": gen_iati21a22,
    "aac5p3": gen_aac5p3,
    "ucrpc21e": gen_ucrpc21e,
    "noimps23p1": gen_noimps23p1,
    "noip10p1": gen_noip10p1,
    "dmopc20c4p4": gen_dmopc20c4p4,
    "996warmup8p3": gen_996warmup8p3,
}


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

    def unlimited_stack():
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))

    rng = random.Random(20261120 + sum(ord(c) for c in name) * 31)
    gen = GENS[name]
    for i in range(iters):
        inp = gen(rng, i)
        ref = subprocess.run([exe], input=inp, capture_output=True, text=True, timeout=10, preexec_fn=unlimited_stack)
        if ref.returncode != 0:
            return False, f"ref crashed on case {i}: {ref.stderr}\ninput:\n{inp}"
        ours = subprocess.run(
            ["python3", CP, "lib-run", port],
            input=inp, capture_output=True, text=True, cwd=ROOT, timeout=60,
        )
        if ours.returncode != 0:
            return False, f"lib-run failed on case {i}:\n{ours.stderr.strip()}\ninput:\n{inp}"
        if ours.stdout.split() != ref.stdout.split():
            return False, (
                f"MISMATCH on case {i}:\ninput:\n{inp}"
                f"ours: {ours.stdout.strip()}\nref:  {ref.stdout.strip()}"
            )
    return True, f"OK ({iters}/{iters} cases match {os.path.basename(orig)})"


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(GENS.keys())
    iters = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    failed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            ok, msg = check_one(name, iters, tmp)
            print(f"[{'PASS' if ok else 'FAIL'}] {name:12s} {msg}", flush=True)
            failed += not ok
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
