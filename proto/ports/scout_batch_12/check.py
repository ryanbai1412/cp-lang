#!/usr/bin/env python3
"""Check the batch-12 scout ports against the original submissions.

Each port is built by `proto/cpc` from a copy, the emitted C++ and the
original named on the port's `// original:` line are compiled, and both
are run on the small random inputs generated below. `cco02p3` is
interactive, so both sides talk to a judge that holds a hidden room.
`cco19p1` is a sketch and is not checked.

    python3 proto/ports/scout_batch_12/check.py               # every port, 200 inputs each
    python3 proto/ports/scout_batch_12/check.py noi05p1 50    # one port, 50 inputs
"""
import os, random, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def grid(rows):
    return "\n".join("".join(r) for r in rows) + "\n"


def gen_ccc18s3():
    n, m = random.randint(4, 8), random.randint(4, 8)
    g = [["W"] * m for _ in range(n)]
    for r in range(1, n - 1):
        for c in range(1, m - 1):
            g[r][c] = random.choice("....WCLRUD.")
    g[random.randint(1, n - 2)][random.randint(1, m - 2)] = "S"
    return f"{n} {m}\n" + grid(g)


def gen_coci10c7p4():
    n = random.randint(2, 6)
    g = [[random.choice("..K") for _ in range(n)] for _ in range(n)]
    pr, pc = random.randrange(n), random.randrange(n)
    g[pr][pc] = "P"
    if not any("K" in r for r in g):
        g[(pr + 1) % n][pc] = "K"
    top = random.choice([3, 10, 10 ** 6])
    high = "\n".join(" ".join(str(random.randint(1, top)) for _ in range(n)) for _ in range(n))
    return f"{n}\n" + grid(g) + high + "\n"


def gen_wossoly21s3():
    n, m = random.randint(1, 6), random.randint(1, 6)
    while n * m < 5:
        n, m = random.randint(1, 6), random.randint(1, 6)
    g = [[random.choice("...#") for _ in range(m)] for _ in range(n)]
    free = [(r, c) for r in range(n) for c in range(m)]
    random.shuffle(free)
    for period in "01234":
        r, c = free.pop()
        g[r][c] = period
    for letter in "abc":
        if len(free) >= 3 and random.random() < 0.6:
            r, c = free.pop()
            g[r][c] = letter
            for _ in range(random.randint(1, 2)):
                r, c = free.pop()
                g[r][c] = letter.upper()
    return f"{n} {m}\n" + grid(g)


def gen_dmopc21c6p3():
    n, m, k = random.randint(1, 6), random.randint(1, 6), random.randint(0, 6)
    rows = [" ".join(str(random.choice([0, 0, 0, random.randint(1, 5)])) for _ in range(m)) for _ in range(n)]
    return f"{n} {m} {k}\n" + "\n".join(rows) + "\n"


def gen_noip09p4():
    g = [[(3 * (r % 3) + r // 3 + c) % 9 + 1 for c in range(9)] for r in range(9)]
    relabel = random.sample(range(1, 10), 9)
    order = [3 * band + r for band in random.sample(range(3), 3) for r in random.sample(range(3), 3)]
    g = [[relabel[v - 1] for v in g[r]] for r in order]
    for _ in range(random.randint(0, 45)):
        g[random.randrange(9)][random.randrange(9)] = 0
    if random.random() < 0.15:                       # sometimes a clash, so the answer is -1
        g[random.randrange(9)][random.randrange(9)] = random.randint(1, 9)
    return "\n".join(" ".join(map(str, r)) for r in g) + "\n"


def gen_noi05p1():
    n, m, q = random.randint(1, 6), random.randint(1, 6), random.randint(1, 5)
    g = [[random.choice("...x") for _ in range(m)] for _ in range(n)]
    x, y = random.randrange(n), random.randrange(m)
    g[x][y] = "."
    out, t = f"{n} {m} {x + 1} {y + 1} {q}\n" + grid(g), 1
    for _ in range(q):
        end = t + random.randint(0, 4)
        out += f"{t} {end} {random.randint(1, 4)}\n"
        t = end + 1
    return out


def gen_ioi05p1():
    n, m, q = random.randint(1, 6), random.randint(1, 6), random.randint(2, 10)
    k = random.randint(1, min(q, 3))
    return f"{n} {m}\n{q} {k}\n" + "".join(f"{random.randint(1, n)} {random.randint(1, m)}\n" for _ in range(q))


def gen_ccc10s4():
    # A block of square pens, so the pens are always joined through inner walls:
    # the original prints a forest's weight when they are not (see the backlog).
    rows, cols = random.randint(1, 3), random.randint(1, 4)
    corner = lambda r, c: r * (cols + 1) + c + 1
    cost = {}
    out = [str(rows * cols)]
    for r in range(rows):
        for c in range(cols):
            ring = [corner(r, c), corner(r, c + 1), corner(r + 1, c + 1), corner(r + 1, c)]
            turn = random.randrange(4)
            ring = ring[turn:] + ring[:turn]
            walls = [cost.setdefault(frozenset((ring[j], ring[(j + 1) % 4])), random.randint(0, 20)) for j in range(4)]
            out.append(" ".join(map(str, [4] + ring + walls)))
    return "\n".join(out) + "\n"


GENS = {name[4:]: f for name, f in list(globals().items()) if name.startswith("gen_")}
MOVES = [(-1, 0), (0, 1), (1, 0), (0, -1)]           # N E S W


def room():
    n, m = random.randint(1, 7), random.randint(1, 7)
    g = [[random.choice("..X") for _ in range(m)] for _ in range(n)]
    x, y = random.randrange(n), random.randrange(m)
    g[x][y] = "."
    return n, m, g, x, y, random.randrange(4)


def reachable(n, m, g, x, y):
    seen, todo = {(x, y)}, [(x, y)]
    while todo:
        r, c = todo.pop()
        for dr, dc in MOVES:
            q = (r + dr, c + dc)
            if 0 <= q[0] < n and 0 <= q[1] < m and g[q[0]][q[1]] == "." and q not in seen:
                seen.add(q)
                todo.append(q)
    return ["".join("." if (r, c) in seen else "X" for c in range(m)) for r in range(n)]


def explore(exe, n, m, g, x, y, d):
    """Play the judge of cco02p3 and return the map the program prints."""
    p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    try:
        p.stdin.write(f"{n} {m} {x + 1} {y + 1} {'NESW'[d]}\n")
        p.stdin.flush()
        for _ in range(100 * n * m):
            line = p.stdout.readline().strip()
            if line == "L":
                d = (d + 3) % 4
            elif line == "R":
                d = (d + 1) % 4
            elif line == "F":
                r, c = x + MOVES[d][0], y + MOVES[d][1]
                moved = 0 <= r < n and 0 <= c < m and g[r][c] == "."
                if moved:
                    x, y = r, c
                p.stdin.write(f"{int(moved)}\n")
                p.stdin.flush()
            elif line == "A":
                return [p.stdout.readline().strip() for _ in range(n)]
            else:
                return ["unexpected: " + line]
        return ["too many commands"]
    finally:
        p.kill()
        p.wait()


def check(name, iters, tmp):
    src = os.path.join(HERE, name + ".cp")
    orig = os.path.join(ROOT, re.search(r"// original: (\S+)", open(src).read()).group(1))
    port = os.path.join(tmp, name + ".cp")           # `build` stamps the file, so work on a copy
    shutil.copy(src, port)
    r = subprocess.run([sys.executable, CP, "build", port], capture_output=True, text=True)
    if r.returncode or not os.path.exists(port[:-3] + ".cpp"):
        return "does not build: " + (r.stdout + r.stderr).strip().splitlines()[-1]
    ref = os.path.join(tmp, name + "_ref.cpp")       # two originals open a local file; read stdin instead
    open(ref, "w").write(open(orig).read().replace('freopen("input.txt", "r", stdin);', ""))
    for cpp, exe in ((port[:-3] + ".cpp", "ours"), (ref, "ref")):
        subprocess.run(["g++", "-std=c++17", "-O2", "-w", "-o", os.path.join(tmp, name + "_" + exe), cpp], check=True)
    ours, ref = (os.path.join(tmp, name + "_" + e) for e in ("ours", "ref"))
    random.seed(12)
    for i in range(iters):
        if name == "cco02p3":
            case = room()
            want = reachable(*case[:5])
            for who, exe in (("port", ours), ("original", ref)):
                if explore(exe, *case) != want:
                    return f"WRONG map from the {who}, conversation {i + 1}: {case}"
            continue
        inp = GENS[name]()
        a, b = (subprocess.run([exe], input=inp, capture_output=True, text=True) for exe in (ours, ref))
        if a.returncode or a.stdout.split() != b.stdout.split():
            return f"WRONG on input {i + 1}:\n{inp}port: {a.stdout}{a.stderr}original: {b.stdout}"
    return f"{'judge' if name == 'cco02p3' else 'stress'} OK: {iters} tests agree"


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else sorted(list(GENS) + ["cco02p3"])
    iters = int(sys.argv[2]) if len(sys.argv) > 2 else 200
    failed = False
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            result = check(name, iters, tmp)
            failed |= "OK" not in result
            print(f"{name}: {result}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
