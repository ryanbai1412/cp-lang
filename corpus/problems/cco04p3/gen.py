"""gen.py SEED SIZE | --max: CCC 2004 Stage 2 P3 Schonfinkel. Lines of S/K trees, each at most
1000 characters, ending with a blank line; every tree has a normal form. Lines are Church
arithmetic from the statement (sums, products and factorials under its N, scaled by size) and
random small trees kept only when a bounded reduction reaches a normal form. The statement does
not bound the number of lines; --max is 60 lines of the heaviest arithmetic that fits in 1000
characters (4!, and products a * b with the largest a * b)."""
import random, sys
sys.setrecursionlimit(10000)
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])

ZERO, SUCC = "(K((SK)K))", "(S((S(KS))K))"
PLUS, TIMES = "((S((SK)K))(K(S((S(KS))K))))", "((S(KS))K)"
NORM = "((S((S((SK)K))(K(S((S(KS))K)))))(K(K((SK)K))))"
FACT = ("((((SS)K)((S(K((SS)(S((SS)K)))))K))((S(K(S((S((S((S((SK)K))(K(K(K((SK)K))))))"
        "(KK)))(K((S((S(KS))K))(K((SK)K))))))))((S(K(S((S(KS))K))))((S((S(KS))K))"
        "(K((S(K(S(K(S(K((S((SK)K))(K(K((SK)K))))))))))((S((S(KS))((S(K(S(KS))))"
        "((S(K(S(KK))))((S((S(KS))K))(K((S((S(KS))((S(K(S(K((S((S(KS))((S(KK))((S(KS))"
        "((S(K(S((SK)K))))K)))))(KK))))))((S((S(KS))K))(K((S((SK)K))(KK)))))))"
        "(K((S((SK)K))(KK))))))))))(K(K((S((S((S(KS))((S(KK))((S(KS))"
        "((S(K(S((SK)K))))K)))))(KK)))((SK)K)))))))))))")


def num(n):
    t = ZERO
    for _ in range(n):
        t = f"({SUCC}{t})"
    return t


def ap(a, b):
    return f"({a}{b})"


def parse(s, i=0):
    if s[i] == "(":
        a, i = parse(s, i + 1)
        b, i = parse(s, i)
        return (a, b), i + 1
    return s[i], i + 1


def size(t):
    return 1 if isinstance(t, str) else size(t[0]) + size(t[1])


def step(t):
    if isinstance(t, tuple):
        f, b = t
        if isinstance(f, tuple) and f[0] == "K":
            return f[1]
        if isinstance(f, tuple) and isinstance(f[0], tuple) and f[0][0] == "S":
            return ((f[0][1], b), (f[1], b))
        for k in (0, 1):
            n = step(t[k])
            if n is not None:
                return (n, b) if k == 0 else (f, n)
    return None


def normalises(s):
    t = parse(s)[0]
    for _ in range(300):
        t = step(t)
        if t is None:
            return True
        if size(t) > 400:
            return False
    return False


def rand_tree(leaves):
    if leaves == 1:
        return r.choice("SK")
    k = r.randint(1, leaves - 1)
    return ap(rand_tree(k), rand_tree(leaves - k))


def arith():
    kind = r.randrange(3)
    if kind == 0:
        a, b = r.randint(0, 1 + S // 2), r.randint(0, 1 + S // 2)
        return ap(NORM, ap(ap(PLUS, num(a)), num(b)))
    if kind == 1:
        a, b = r.randint(0, 1 + S // 4), r.randint(0, 1 + S // 4)
        return ap(NORM, ap(ap(TIMES, num(a)), num(b)))
    return ap(NORM, ap(FACT, num(r.randint(0, min(4, S // 8)))))


if MAX:
    fits = [(a * b, a, b) for a in range(60) for b in range(60)
            if len(ap(NORM, ap(ap(TIMES, num(a)), num(b)))) <= 1000]
    best = sorted(fits, reverse=True)[:30]
    lines = [ap(NORM, ap(FACT, num(4)))] * 30
    lines += [ap(NORM, ap(ap(TIMES, num(a)), num(b))) for _, a, b in best]
else:
    lines = []
    for _ in range(r.randint(1, 2 + S // 4)):
        if r.random() < 0.3:
            lines.append(arith())
        else:
            while not normalises(t := rand_tree(r.randint(1, 3 + S))):
                pass
            lines.append(t)
assert all(len(x) <= 1000 for x in lines)
print("\n".join(lines))
print()
