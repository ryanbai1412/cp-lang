"""gen.py SEED SIZE | --max: WC 2022 P1. A test point id (1..20) fixes n's bound, k and whether
operations 5 and 6 are unlimited (the statement's table); T <= 3 cases of n <= min(bound, SIZE)
pairs, each sequence uniform, a left chain, a right chain or nested runs. --max is test point 20:
T = 3, n = 1e5, k = 3e5, on the shapes that need the most operations."""
import random, sys

ROWS = [(3, 10, 100000), (6, 100, 10000), (8, 500, 100000), (11, 1000, 10000),
        (13, 5000, 20000), (16, 100000, 300000), (20, 100000, 300000)]


def row(tid):
    return next((bound, k) for last, bound, k in ROWS if tid <= last)


def uniform(r, n):
    # A uniform Dyck word: the cycle lemma on n '(' and n + 1 ')'.
    w = [1] * n + [-1] * (n + 1)
    r.shuffle(w)
    low, at, d = 0, 0, 0
    for i, x in enumerate(w):
        d += x
        if d < low:
            low, at = d, i + 1
    w = w[at:] + w[:at]
    return "".join("(" if x > 0 else ")" for x in w[:-1])


def shape(r, n):
    kind = r.randrange(5)
    if kind == 0:
        return "(" * n + ")" * n
    if kind == 1:
        return "()" * n
    if kind == 2:
        out, left = [], n
        while left:
            run = r.randint(1, left)
            out.append("(" * run + ")" * run)
            left -= run
        return "".join(out)
    return uniform(r, n)


if sys.argv[1] == "--max":
    print(20, 3)
    for s1, s2 in [("(" + "()" * 99999 + ")", "(" * 100000 + ")" * 100000),
                   ("()" + "(" * 99999 + ")" * 99999, "(" * 100000 + ")" * 100000),
                   (uniform(random.Random(0), 100000), "(" * 100000 + ")" * 100000)]:
        print(100000, 300000)
        print(s1)
        print(s2)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
assert size >= 1
tid = r.randint(1, 20)
bound, k = row(tid)
t = r.randint(1, 3)
print(tid, t)
for _ in range(t):
    n = r.randint(1, max(1, min(bound, size)))
    print(n, k)
    print(shape(r, n))
    print(shape(r, n))
