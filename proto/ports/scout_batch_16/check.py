#!/usr/bin/env python3
"""Check the batch-16 scout ports against the original submissions.

Each runnable port runs on the reference interpreter (`proto/cp.py lib-run`)
and the original is compiled from `corpus/submissions/<id>.cpp`; both get
the same small random inputs.

Every generator here is constructive: it builds an input that satisfies the
statement's promise (walls that bound real regions, a square that can be
completed, an equation that parses) instead of drawing one and rejecting
it. They are the Python stand-ins for the `gen` hooks the friction backlog
asks for.

ccc19s3 has many right answers and both programs are randomized, so each
output is validated against the statement instead (see `judge_ccc19s3`).

    python3 proto/ports/scout_batch_16/check.py                # every port, 60 inputs each
    python3 proto/ports/scout_batch_16/check.py noi03p1 200    # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def gen_noip07p2():
    n = R(1, 14)
    word = "".join(random.choice("ace19-z3-b-") for _ in range(n))
    return lines([(R(1, 3), R(1, 3), R(1, 2)), (word,)])


def gen_ccc00s5():
    # Distinct sheep, two decimals, strictly above the fence. A small range
    # makes equal distances and sheep straight above each other common.
    n = R(1, 6)
    sheep = set()
    while len(sheep) < n:
        sheep.add((R(0, 12) * 25, R(1, 12) * 25))
    return "%d\n" % n + "".join("%.2f\n%.2f\n" % (x / 100, y / 100) for x, y in sheep)


def gen_cco05p4():
    # Even-heavy sequences have few prime sums, so longer runs and
    # "anti-primed" both come up.
    t = R(1, 4)
    rows = [(t,)]
    for _ in range(t):
        n = R(1, 8)
        step = random.choice([1, 2, 2, 6])
        rows.append([n] + [R(1, 12) * step for _ in range(n)])
    return lines(rows)


def gen_coci21c6p5():
    # Pairs partition the 2n positions and are given as x < y. Half the
    # cases start from a solvable instance: a balanced word of n brackets,
    # each matched with a decoy bracket.
    rows = []
    t = R(1, 4)
    for _ in range(t):
        n = R(1, 5)
        pos = list(range(1, 2 * n + 1))
        random.shuffle(pos)
        pairs = sorted(tuple(sorted(pos[2 * i:2 * i + 2])) for i in range(n))
        s = [random.choice("()") for _ in range(2 * n)]
        if R(0, 1) and n % 2 == 0:
            kept = sorted(random.choice(p) for p in pairs)
            depth = 0
            for i, k in enumerate(kept):
                left = len(kept) - i
                c = "(" if depth == 0 or (depth < left and R(0, 1)) else ")"
                depth += 1 if c == "(" else -1
                s[k - 1] = c
        random.shuffle(pairs)
        rows += [(n,), ("".join(s),)] + pairs
    return lines([(t,)] + rows)


def gen_ccc19s3():
    # Guaranteed solvable: start from a full arithmetic square, hide cells.
    a, dr, dc, dd = R(-20, 20), R(-6, 6), R(-6, 6), R(-3, 3)
    sq = [[a + i * dr + j * dc + i * j * dd for j in range(3)] for i in range(3)]
    hide = R(0, 9)
    for i, j in random.sample([(i, j) for i in range(3) for j in range(3)], hide):
        sq[i][j] = "X"
    return lines(sq)


def judge_ccc19s3(inp, out):
    given = [l.split() for l in inp.splitlines()]
    try:
        got = [[int(x) for x in l.split()] for l in out.splitlines()]
    except ValueError:
        return False
    if len(got) != 3 or any(len(r) != 3 for r in got):
        return False
    for i in range(3):
        for j in range(3):
            if given[i][j] != "X" and int(given[i][j]) != got[i][j]:
                return False
    rows_ok = all(r[1] - r[0] == r[2] - r[1] for r in got)
    cols_ok = all(got[1][j] - got[0][j] == got[2][j] - got[1][j] for j in range(3))
    return rows_ok and cols_ok


def gen_noi03p1():
    # A well-formed equation: numbers without leading zeros, one '='. Half
    # the cases are a true equation with one match moved, so "No" is not the
    # only answer that gets exercised.
    def num():
        return str(R(0, 9) if R(0, 1) else R(10, 120))

    def side():
        return num() + "".join(random.choice("+-") + num() for _ in range(R(0, 2)))

    if R(0, 1):
        a, b = R(0, 60), R(0, 60)
        eq = list(random.choice(["%d+%d=%d" % (a, b, a + b), "%d=%d-%d" % (a, a + b, b)]))
        swaps = {"0": "69", "6": "09", "9": "06", "2": "3", "3": "25", "5": "3"}
        spots = [i for i, c in enumerate(eq) if c in swaps]
        if spots:
            i = random.choice(spots)
            eq[i] = random.choice(swaps[eq[i]])
        text = "".join(eq)
        # A reshaped leading digit must not become a leading zero.
        if any(tok[0] == "0" and len(tok) > 1 for tok in text.replace("=", "+").replace("-", "+").split("+")):
            text = "%d+%d=%d" % (a, b, a + b + 1)
        return text + "#\n"
    return side() + "=" + random.choice(["", "-"]) + side() + "#\n"


def gen_ioi00p4():
    # A real planar subdivision, built rather than filtered: an a x b
    # lattice of towns whose unit squares are the inner regions, plus the
    # outer region walked round the rim. Merging two squares across a wall
    # would stay planar too; squares already give regions that share walls,
    # towns on many regions, and an outer face.
    a, b = R(1, 3), R(1, 3)
    town = lambda i, j: i * (b + 1) + j + 1
    regions = []
    for i in range(a):
        for j in range(b):
            regions.append([town(i, j), town(i, j + 1), town(i + 1, j + 1), town(i + 1, j)])
    rim = [town(0, j) for j in range(b)] + [town(i, b) for i in range(a)]
    rim += [town(a, j) for j in range(b, 0, -1)] + [town(i, 0) for i in range(a, 0, -1)]
    regions.append(rim)
    random.shuffle(regions)
    n = (a + 1) * (b + 1)
    k = R(1, min(n, 5))
    homes = random.sample(range(1, n + 1), k)
    rows = [(len(regions),), (n,), (k,), homes]
    for reg in regions:
        s = R(0, len(reg) - 1)
        rows += [(len(reg),), reg[s:] + reg[:s]]
    return lines(rows)


PORTS = {
    "noip07p2": ("5184882", gen_noip07p2, None),
    "ccc00s5": ("4584542", gen_ccc00s5, None),
    "cco05p4": ("4665767", gen_cco05p4, None),
    "coci21c6p5": ("4531869", gen_coci21c6p5, None),
    "ccc19s3": ("4325116", gen_ccc19s3, judge_ccc19s3),
    "noi03p1": ("3738358", gen_noi03p1, None),
    "ioi00p4": ("4682453", gen_ioi00p4, None),
}


def run(cmd, inp):
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=60)
    return r.stdout if r.returncode == 0 else "<exit %d> %s" % (r.returncode, r.stderr[-300:])


def check(name, cases, tmp):
    sub, gen, judge = PORTS[name]
    exe = os.path.join(tmp, name)
    src = os.path.join(ROOT, "corpus", "submissions", sub + ".cpp")
    subprocess.run(["g++", "-std=c++20", "-O1", "-w", "-o", exe, src], check=True)
    port = [sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", os.path.join(HERE, name + ".cp")]
    for i in range(cases):
        inp = gen()
        want, got = run([exe], inp), run(port, inp)
        ok = (judge(inp, want) and judge(inp, got)) if judge else want.split() == got.split()
        if not ok:
            print("%s: MISMATCH on case %d\n--- input\n%s--- original\n%s--- port\n%s" % (name, i, inp, want, got))
            return False
    print("%s: %d cases agree" % (name, cases))
    return True


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or list(PORTS)
    cases = next((int(a) for a in sys.argv[1:] if a.isdigit()), 60)
    random.seed(16)
    with tempfile.TemporaryDirectory() as tmp:
        ok = all([check(n, cases, tmp) for n in names])
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
