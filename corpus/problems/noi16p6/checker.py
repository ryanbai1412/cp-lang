"""noi16p6 checker: `checker.py input output answer`. The output is a computer for the task in the
input; it is run on seeded random inputs (plus edge values) with every node rounded to 90 decimal
places, and passes when it stays within 10^4 nodes and 10^1000 in magnitude and every output is
within 10^-9 of the task's value. The answer file is not consulted: any correct computer passes."""
import random, sys
from decimal import Decimal as D, getcontext, ROUND_HALF_EVEN

getcontext().prec = 1300
Q = D(10) ** -90
LIM = D(10) ** 1000
ARGS = {"I": 0, "O": 1, "+": 2, "C": 2, "-": 1, "<": 2, ">": 2, "S": 1, "P": 2, "M": 2, "*": 2}


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def rnd(x):
    return x.quantize(Q, rounding=ROUND_HALF_EVEN)


def sig(x):
    if x > 300:
        return D(1)
    if x < -300:
        return D(0)
    return 1 / (1 + (-x).exp())


def real(r):
    return D(r.randint(-10**18, 10**18)) / D(10**9)


def task(t, r, edge):
    """(inputs, expected outputs)"""
    if t in (1,):
        a, b = (D(10**9), D(-10**9)) if edge else (real(r), real(r))
        return [a, b], [-2 * a - 2 * b]
    if t in (2, 3, 4, 8):
        a = [D(0), D(10**9), D(-10**9), D("0.000000001"), D("-0.000000001")][edge % 5] if edge else real(r)
        if t == 2:
            getcontext().prec = 1300
            return [a], [sig(-17 * a)]
        if t == 3:
            return [a], [D((a > 0) - (a < 0))]
        if t == 4:
            return [a], [abs(a)]
        return [a], [a / 10]
    if t == 5:
        a = [D(r.randint(0, 1)) for _ in range(32)] if not edge else [D(edge % 2)] * 32
        v = 0
        for x in a:
            v = 2 * v + int(x)
        return a, [D(v)]
    if t == 6:
        a = [0, 2**32 - 1, 1, 2**31][edge % 4] if edge else r.randrange(2**32)
        return [D(a)], [D(a >> i & 1) for i in range(31, -1, -1)]
    if t == 7:
        a, b = ((2**32 - 1, 0), (2**32 - 1, 2**32 - 1), (0, 0))[edge % 3] if edge else (r.randrange(2**32), r.randrange(2**32))
        return [D(a), D(b)], [D(a ^ b)]
    if t == 9:
        a = [real(r) for _ in range(16)]
        if edge:
            a = [D(r.randint(-2, 2)) for _ in range(16)]
        return a, sorted(a)
    a, b, m = ((2**32 - 1, 2**32 - 1, 1), (2**32 - 1, 2**32 - 1, 2**32 - 1), (0, 5, 3), (2**32 - 1, 2**32 - 2, 2**31 + 7))[edge % 4] if edge else (r.randrange(2**32), r.randrange(2**32), r.randrange(1, 2**32))
    return [D(a), D(b), D(m)], [D(a * b % m)]


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    t = int(open(sys.argv[1]).read().split()[0])
    lines = [l.split() for l in open(sys.argv[2]).read().splitlines() if l.strip()]
    if not lines or len(lines) > 10**4:
        fail(f"{len(lines)} nodes, want 1..10^4")
    prog = []
    for i, w in enumerate(lines, 1):
        if w[0] not in ARGS or len(w) != 1 + ARGS[w[0]]:
            fail(f"node {i}: bad node {' '.join(w)}")
        try:
            refs = [int(x) for x in w[1:2]] + ([int(w[2])] if w[0] in "+PM*" else [])
            k = D(w[2]) if w[0] == "C" else int(w[2]) if w[0] in "<>" else None
        except Exception:
            fail(f"node {i}: bad operand")
        if any(not 1 <= j < i for j in refs):
            fail(f"node {i}: operand out of range")
        if w[0] == "C" and rnd(k) != k:
            fail(f"node {i}: c has more than 90 decimal places")
        if w[0] in "<>" and not 0 <= k <= 10000:
            fail(f"node {i}: bad shift")
        prog.append((w[0], refs, k))
    r = random.Random(16)
    for trial in range(40):
        ins, want = task(t, r, trial % 4 == 0 and trial // 4 + 1)
        x, got, it = [], [], iter(ins)
        for op, f, k in prog:
            a = x[f[0] - 1] if f else None
            b = x[f[1] - 1] if len(f) > 1 else None
            if op == "I":
                v = next(it, None)
                if v is None:
                    fail("more input nodes than inputs")
            elif op == "O": v = a; got.append(a)
            elif op == "+": v = a + b
            elif op == "C": v = a + k
            elif op == "-": v = -a
            elif op == "<": v = a * D(2) ** k
            elif op == ">": v = a / D(2) ** k
            elif op == "S": v = sig(a)
            elif op == "P": v = D((a > b) - (a < b))
            elif op == "M": v = a if a > b else b
            else: v = a * b
            v = rnd(v)
            if abs(v) > LIM:
                fail("a node exceeds 10^1000")
            x.append(v)
        if next(it, None) is not None:
            fail("fewer input nodes than inputs")
        if len(got) != len(want):
            fail(f"{len(got)} outputs, want {len(want)}")
        for g, e in zip(got, want):
            if abs(g - e) > D(10) ** -9:
                fail(f"inputs {[str(v) for v in ins]}: output {g}, want {e}")

main()
