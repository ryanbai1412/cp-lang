#!/usr/bin/env python3
"""Check the batch-7 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs and must print the same output. cco17p3 is also checked
against a brute force, which stands in for the original on the inputs where
the original is undefined (see `original_is_defined_cco17p3`).

    python3 proto/ports/scout_batch_7/check.py                # every port, 60 inputs each
    python3 proto/ports/scout_batch_7/check.py noi04p1 200    # one port, 200 inputs
"""
import os, random, string, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def gen_ioi01p1():
    n = R(1, 6)
    rows = [(0, n)]
    for _ in range(R(1, 25)):
        if R(0, 1):
            rows.append((1, R(0, n - 1), R(0, n - 1), R(-5, 9)))
        else:
            (l, r), (b, t) = sorted((R(0, n - 1), R(0, n - 1))), sorted((R(0, n - 1), R(0, n - 1)))
            rows.append((2, l, b, r, t))
    return lines(rows + [(3,)])


def gen_noi01p1():
    n, k = R(1, 7), R(1, 25)
    return lines([(n, k)] + [(R(1, 2), R(1, n + 1), R(1, n + 1)) for _ in range(k)])


def gen_noi04p1():
    q, floor = R(1, 30), R(0, 20)
    rows = [(q, floor)]
    for _ in range(q):
        op = random.choice("IIIASFF")
        rows.append((op, R(0, 40) if op == "I" else R(1, 8)))
    return lines(rows)


def gen_noi03p2():
    # Text without spaces: the interpreter reads whitespace-separated tokens only.
    alphabet = string.ascii_letters + string.digits + "+-*/"
    rows, size, cursor = [], 0, 0
    for _ in range(R(1, 30)):
        op = random.choice(["Move", "Insert", "Insert", "Delete", "Get", "Get", "Prev", "Next"])
        if op == "Move":
            cursor = R(0, size)
            rows.append((op, cursor))
        elif op == "Insert":
            n = R(1, 8)
            text = "".join(random.choice(alphabet) for _ in range(n))
            cutpoint = R(1, n)
            rows.append((op, n))
            rows.append((text[:cutpoint],))
            if cutpoint < n:
                rows.append((text[cutpoint:],))
            size += n
        elif op in ("Delete", "Get") and size > cursor:
            n = R(1, size - cursor)
            rows.append((op, n))
            size -= n if op == "Delete" else 0
        elif op == "Prev" and cursor > 0:
            cursor -= 1
            rows.append((op,))
        elif op == "Next" and cursor < size:
            cursor += 1
            rows.append((op,))
    return lines([(len([r for r in rows if r[0] in ("Move", "Insert", "Delete", "Get", "Prev", "Next")]),)] + rows)


def gen_ccc14s4hard():
    n = R(1, 12)
    rows = [(n, R(1, 8))]
    for _ in range(n):
        x1, y1 = R(0, 12), R(0, 12)
        rows.append((x1, y1, x1 + R(1, 8), y1 + R(1, 8), R(1, 4)))
    return lines(rows)


def gen_tsoc15c2p6():
    n, q = R(1, 9), R(1, 12)
    rows = [(n, q), [R(0, 30) for _ in range(n)]]
    for _ in range(q):
        l, r = sorted((R(1, n), R(1, n)))
        rows.append((l, r, R(0, 12)))
    return lines(rows)


def gen_cco17p3():
    n, q = R(1, 12), R(1, 12)
    big = random.choice([6, 6, 40, 10 ** 12, 10 ** 18])
    return lines([(n, q)] + [(R(1, big), R(1, big), R(1, 9)) for _ in range(n)] + [(R(1, big), R(1, big)) for _ in range(q)])


def gen_ioi12p3():
    rows, commands, size_after = [], 0, [0]
    for _ in range(R(1, 30)):
        op = random.choice("TTTUPP")
        if op == "T":
            rows.append((op, random.choice("abcde")))
            size_after.append(size_after[-1] + 1)
        elif op == "U" and commands > 0:
            x = R(1, commands)
            rows.append((op, x))
            size_after.append(size_after[-1 - x])
        elif op == "P" and size_after[-1] > 0:
            rows.append((op, R(0, size_after[-1] - 1)))
            continue
        else:
            continue
        commands += 1
    return lines([(len(rows),)] + rows)


def gen_coci20c6p5():
    n, q = R(1, 12), R(1, 12)
    rows = [(n, q), [R(1, random.choice([3, n, 2 * n])) for _ in range(n)]]
    for _ in range(q):
        rows.append(sorted((R(1, n), R(1, n))))
    return lines(rows)


def gen_sac22cc5p4():
    n, q = R(1, 8), R(1, 12)
    rows = [(n, q)]
    for _ in range(n + q):
        l = R(1, 30)
        rows.append((l, l + R(0, 6)))
    return lines(rows)


def brute_cco17p3(data):
    # Straight from the statement: a stroke at x reaches row r when r is x plus a multiple of x's top bit.
    rows = [list(map(int, l.split())) for l in data.splitlines()]
    n, strokes, cells = rows[0][0], rows[1:rows[0][0] + 1], rows[rows[0][0] + 1:]
    top = lambda x: 1 << (x.bit_length() - 1)
    hits = lambda x, r: r >= x and (r - x) % top(x) == 0
    return [str(sum(v for x, y, v in strokes if hits(x, r) and hits(y, c))) for r, c in cells]


def original_is_defined_cco17p3(data):
    # The original reserves 110 trie nodes per stroke and keeps references into the
    # vector while it grows. A stroke near 10^18 can need 119, and then it reads
    # freed memory (it aborts on glibc). Up to 10^12 a stroke needs at most 79.
    return max(int(t) for t in data.split()) <= 10 ** 12


# name -> (answers from the statement, whether the original may be run on this input)
BRUTE = {"cco17p3": (brute_cco17p3, original_is_defined_cco17p3)}


# The Scrivener original is a set of functions for the IOI grader. This is the
# driver: the number of lines, then "T c", "U x" or "P i" on each.
MAIN_SCRIVENER = r"""
int main() {
    int n;
    scanf("%d", &n);
    Init();
    while (n--) {
        char op;
        scanf(" %c", &op);
        if (op == 'T') { char c; scanf(" %c", &c); TypeLetter(c); }
        else if (op == 'U') { int x; scanf("%d", &x); UndoCommands(x); }
        else { int i; scanf("%d", &i); printf("%c\n", GetLetter(i)); }
    }
}
"""

PORTS = {
    "ioi01p1": (3477640, gen_ioi01p1, ""),
    "noi01p1": (6454234, gen_noi01p1, ""),
    "noi04p1": (3738402, gen_noi04p1, ""),
    "noi03p2": (5032747, gen_noi03p2, ""),
    "ccc14s4hard": (5672273, gen_ccc14s4hard, ""),
    "tsoc15c2p6": (3657404, gen_tsoc15c2p6, ""),
    "cco17p3": (4843963, gen_cco17p3, ""),
    "ioi12p3": (4682653, gen_ioi12p3, MAIN_SCRIVENER),
    "coci20c6p5": (4188128, gen_coci20c6p5, ""),
    "sac22cc5p4": (4575559, gen_sac22cc5p4, ""),
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
        brute, original_ok = BRUTE.get(name, (None, lambda data: True))
        want = subprocess.run([exe], input=data, capture_output=True, text=True).stdout.split() if original_ok(data) else brute(data)
        if brute and brute(data) != want:
            return f"the original disagrees with the statement on input {i}:\n{data}"
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
