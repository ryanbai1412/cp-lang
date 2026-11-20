#!/usr/bin/env python3
"""Differential test harness for Scout Batch 44 (hard DMOPC 2020-2023 classics).

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable CP Lang port in today/ via
`proto/cp.py lib-run` on randomized and edge-case inputs.

    python3 proto/ports/scout_batch_44/check.py                # all 10 ports, 30 cases each
    python3 proto/ports/scout_batch_44/check.py dmopc21c9p3 60 # single port, 60 cases

dmopc21c2p3 is interactive: the original C++ is driven by a local judge (see
`judge_dmopc21c2p3`), while the port embeds the same judge inline. Both must
recover every hidden value within the query limit.
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_dmopc21c8p2(rng, idx):
    if idx == 0:
        return lines((3, 5), [1, 0, 3], [0, 0, 2])
    if idx == 1:
        return lines((1, 1), [0], [0])
    n = rng.randint(1, 12)
    m = rng.randint(1, 9)
    zero = rng.choice([0.2, 0.5, 0.9])
    cell = lambda: 0 if rng.random() < zero else rng.randint(1, m)
    return lines((n, m), [cell() for _ in range(n)], [cell() for _ in range(n)])


def gen_dmopc21c4p4(rng, idx):
    return lines((idx + 2 if idx < 12 else rng.randint(2, 3000),))


def gen_dmopc20c5p2(rng, idx):
    if idx < 3:
        return lines([(1, 1), (4, 6), (7, 7)][idx])
    hi = rng.choice([6, 30, 120])
    return lines((rng.randint(1, hi), rng.randint(1, hi)))


def gen_dmopc21c9p2(rng, idx):
    t = rng.randint(1, 8)
    rows = [(t,)]
    for _ in range(t):
        alpha = rng.choice(["ab", "abc", "yz", "a"])
        a = "".join(rng.choice(alpha) for _ in range(rng.randint(1, 14)))
        if rng.random() < 0.6:
            # Replay the reduction so that many cases are YES instances.
            st, b = [], []
            for c in a:
                st.append(c)
                while len(st) >= 2 and st[-1] == st[-2] and st[-1] != "z":
                    x = st.pop(); st.pop(); st.append(chr(ord(x) + 1))
                while st and rng.random() < 0.4:
                    b.append(st.pop(0))
            b += st
            b = "".join(b) or rng.choice(alpha)
        else:
            b = "".join(rng.choice("abcdz") for _ in range(rng.randint(1, 8)))
        rows.append((a,))
        rows.append((b,))
    return lines(*rows)


def gen_dmopc21c5p3(rng, idx):
    if idx == 0:
        return lines((8,), (1,), (2,), (3,), (4,), (11,), (36,), (311,), (312,))
    t = rng.randint(1, 6)
    return lines((t,), *[(rng.randint(1, 400),) for _ in range(t)])


def gen_dmopc21c4p2(rng, idx):
    if idx == 0:
        return lines((5,), [6, 2, 3, 12, 5])
    n = rng.randint(1, 25)
    hi = rng.choice([12, 60, 600, 20000])  # the interpreter sieve caps the range
    return lines((n,), [rng.randint(1 if rng.random() < 0.1 else 2, hi) for _ in range(n)])


def gen_dmopc21c9p3(rng, idx):
    n = rng.randint(2, 9)
    edges = [(rng.randint(1, v - 1), v) for v in range(2, n + 1)]
    for _ in range(rng.randint(0, n)):
        u, v = rng.sample(range(1, n + 1), 2)
        edges.append((u, v))
    q = rng.randint(1, 12)
    rows = [(n, len(edges))] + edges + [(q,)]
    for _ in range(q):
        rows.append((rng.randint(1, n), rng.randint(1, n), rng.randint(1, n),
                     rng.randint(1, n), rng.randint(1, n), rng.randint(1, n)))
    return lines(*rows)


def gen_dmopc21c7p2(rng, idx):
    n = rng.randint(1, 14)
    q = rng.randint(1, 14)
    order = list(range(1, n + 1))
    rows = [(n, q)]
    for _ in range(q):
        # Track the real order so that every operation is a valid splice.
        i = rng.randint(0, n - 1)
        j = rng.randint(i, n - 1)
        block, rest = order[i:j + 1], order[:i] + order[j + 1:]
        pos = rng.randint(0, len(rest))
        rows.append((block[0], block[-1], rest[pos - 1] if pos else 0))
        order = rest[:pos] + block + rest[pos:]
    return lines(*rows)


def gen_dmopc21c2p3(rng, idx):
    n = rng.randint(1, 8)
    vals = [rng.choice([0, 1, rng.randrange(1 << rng.randint(1, 59)), (1 << 60) - 1])
            for _ in range(n)]
    return lines((n,), vals)


def gen_dmopc21c6p2(rng, idx):
    n = rng.randint(1, 30)
    p = rng.choice([0.0, 0.3, 0.7, 1.0])
    s = "".join("1" if rng.random() < p else "0" for _ in range(n))
    return lines((n, rng.randint(1, n)), (s,))


def judge_dmopc21c2p3(exe, data):
    """Drive the original interactive solution; return the values it pinned down."""
    hidden = list(map(int, data.split()[1:]))
    p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    p.stdin.write(f"{len(hidden)}\n"); p.stdin.flush()
    queries, found = 0, []
    for x in hidden:
        while True:
            g = int(p.stdout.readline())
            queries += 1
            a, b, d = x, g, 0
            while a != b:
                if a > b: a >>= 1
                else: b >>= 1
                d += 1
            p.stdin.write(f"{d}\n"); p.stdin.flush()
            if d == 0:
                found.append(g)
                break
    p.stdin.close()
    assert p.wait(timeout=10) == 0 and queries < 32000
    return "\n".join(map(str, found)) + "\n"


PROBLEMS = [
    ("dmopc21c8p2", "4522631", gen_dmopc21c8p2),
    ("dmopc21c4p4", "4131060", gen_dmopc21c4p4),
    ("dmopc20c5p2", "3591332", gen_dmopc20c5p2),
    ("dmopc21c9p2", "4563973", gen_dmopc21c9p2),
    ("dmopc21c5p3", "4218397", gen_dmopc21c5p3),
    ("dmopc21c4p2", "4125290", gen_dmopc21c4p2),
    ("dmopc21c9p3", "4564027", gen_dmopc21c9p3),
    ("dmopc21c7p2", "4452915", gen_dmopc21c7p2),
    ("dmopc21c2p3", "3946741", gen_dmopc21c2p3),
    ("dmopc21c6p2", "4342045", gen_dmopc21c6p2),
]


def main():
    only = sys.argv[1] if len(sys.argv) > 1 else None
    cases = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    failed = False
    with tempfile.TemporaryDirectory() as tmp:
        for name, sub, gen in PROBLEMS:
            if only and name != only:
                continue
            exe = os.path.join(tmp, name)
            subprocess.run(["g++", "-O2", "-std=c++17", "-w", "-o", exe,
                            os.path.join(ROOT, "corpus", "submissions", sub + ".cpp")], check=True)
            rng = random.Random(name)
            ok = 0
            for idx in range(cases):
                data = gen(rng, idx)
                if name == "dmopc21c2p3":
                    want = judge_dmopc21c2p3(exe, data)
                else:
                    want = subprocess.run([exe], input=data, capture_output=True, text=True, timeout=20).stdout
                got = subprocess.run([sys.executable, CP, "lib-run", os.path.join(HERE, "today", name + ".cp")],
                                     input=data, capture_output=True, text=True, timeout=120)
                if got.returncode != 0 or got.stdout.split() != want.split() or "error" in got.stdout[:6]:
                    failed = True
                    print(f"FAIL {name} case {idx}\n--- input\n{data}--- want\n{want}--- got\n{got.stdout}{got.stderr}")
                    break
                ok += 1
            print(f"{name:14s} {ok}/{cases} ok")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
