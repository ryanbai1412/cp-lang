#!/usr/bin/env python3
"""Check the batch-10 scout ports against the original submissions.

Each port runs on the reference interpreter (`proto/cp.py lib-run`) and the
original is compiled from `corpus/submissions/<id>.cpp`; both get the same
small random inputs and must print the same output. btoi19p5 has many right
answers, so its two outputs are compared by length and each is validated
against the statement instead (see `judge_btoi19p5`).

    python3 proto/ports/scout_batch_10/check.py                # every port, 60 inputs each
    python3 proto/ports/scout_batch_10/check.py noi15p5 200    # one port, 200 inputs
"""
import os, random, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
R = random.randint


def lines(rows):
    return "".join(" ".join(map(str, r)) + "\n" for r in rows)


def word(n, letters="ab"):
    return "".join(random.choice(letters) for _ in range(n))


def periodic(n):
    # Random words rarely repeat; these do.
    unit = word(R(1, 3), random.choice(["a", "ab", "abc"]))
    w = list((unit * n)[:n])
    for _ in range(R(0, 2)):
        w[R(0, n - 1)] = random.choice("abc")
    return "".join(w)


def gen_ctsc06p1():
    m, t = R(1, 5), R(1, 4)
    rows = [(m, t)]
    for _ in range(t):
        n = R(1, 12)
        name = [ord(c) - 96 for c in periodic(n)]
        rows.append([n] + [min(x, m) for x in name])
    return lines(rows)


def gen_noip20p2():
    t = R(1, 4)
    return lines([(t,)] + [(periodic(R(2, 14)),) for _ in range(t)])


def gen_noi15p5():
    n = R(2, 12)
    return lines([(n,), (periodic(n),), [R(-9, 9) for _ in range(n)]])


def gen_saspeed():
    return periodic(R(1, 16)) + "\n"


def gen_noi16p1():
    t = R(1, 4)
    return lines([(t,)] + [(periodic(R(1, 14)),) for _ in range(t)])


def gen_nccc10s3():
    s = random.choice([periodic(R(1, 14)), "a" * R(1, 6)])
    rows = [(s,), (R(1, 8),)]
    for _ in range(rows[1][0]):
        i = R(0, len(s) - 1)
        t = s[i:R(i + 1, len(s))] if R(0, 3) else word(R(1, 3), "abc")
        rows.append((t, R(1, 5)))
    return lines(rows)


def gen_btoi19p5():
    return lines([(word(R(1, 9), "abc"),), (word(R(1, 9), "abc"),)])


def gen_noi17p2():
    n, q = R(1, 8), R(1, 14)
    worms = [R(1, 3) for _ in range(n)]
    rows, nxt, prv = [], [None] * n, [None] * n
    for _ in range(q):
        op = random.choice([1, 1, 2, 3, 3])
        heads = [i for i in range(n) if prv[i] is None]
        tails = [i for i in range(n) if nxt[i] is None]

        def head_of(i):
            while prv[i] is not None:
                i = prv[i]
            return i

        pairs = [(u, v) for u in tails for v in heads if head_of(u) != v]
        if op == 1 and pairs:
            u, v = random.choice(pairs)
            nxt[u], prv[v] = v, u
            rows.append((1, u + 1, v + 1))
        elif op == 2 and any(x is not None for x in nxt):
            u = random.choice([i for i in range(n) if nxt[i] is not None])
            prv[nxt[u]], nxt[u] = None, None
            rows.append((2, u + 1))
        else:
            s = word(R(1, 6), "123")
            rows.append((3, s, R(1, len(s))))
    return lines([(n, len(rows)), worms] + rows)


def gen_year2018p7():
    n = R(1, 6)
    return lines([(n,)] + [(R(1, i),) for i in range(1, n)] + [(word(R(1, 6)),) for _ in range(n)])


def gen_sac22cc3p5():
    n, m = R(1, 6), R(1, 4)
    letters = random.choice(["ab", "abAB"])
    return lines([(n, m)] + [(word(R(1, 4), letters),) for _ in range(n)] + [(word(R(1, 12), letters),) for _ in range(m)])


def judge_btoi19p5(data, out):
    """The length a claimed answer proves, or None if the claim is false: the
    block of s must be a rotation of the block of t, or of its mirror image."""
    s, t = data.split()
    try:
        k, a, b = map(int, out.split())
    except ValueError:
        return None
    if not (0 <= k and 0 <= a and a + k <= len(s) and 0 <= b and b + k <= len(t)):
        return None
    x, y = s[a:a + k], t[b:b + k]
    return k if any(x == y[i:] + y[:i] or x == (y[i:] + y[:i])[::-1] for i in range(max(k, 1))) else None


JUDGE = {"btoi19p5": judge_btoi19p5}

# name -> (original submission, input generator)
PORTS = {
    "ctsc06p1": (4528855, gen_ctsc06p1),
    "noip20p2": (5445673, gen_noip20p2),
    "noi15p5": (3927929, gen_noi15p5),
    "saspeed": (4852075, gen_saspeed),
    "noi16p1": (4797772, gen_noi16p1),
    "nccc10s3": (4214399, gen_nccc10s3),
    "btoi19p5": (4456167, gen_btoi19p5),
    "noi17p2": (5690636, gen_noi17p2),
    "year2018p7": (4899354, gen_year2018p7),
    "sac22cc3p5": (4340958, gen_sac22cc3p5),
}


def check(name, iters, tmp):
    sub, gen = PORTS[name]
    exe = os.path.join(tmp, name + "_orig")
    r = subprocess.run(["g++", "-std=c++20", "-O1", "-w", "-o", exe, os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp")],
                       capture_output=True, text=True)
    if r.returncode:
        return "original does not compile: " + r.stderr[:300]
    random.seed(name)
    for i in range(iters):
        data = gen()
        want = subprocess.run([exe], input=data, capture_output=True, text=True).stdout
        got = subprocess.run([sys.executable, os.path.join(ROOT, "proto", "cp.py"), "lib-run", os.path.join(HERE, name + ".cp")],
                             input=data, capture_output=True, text=True)
        judge = JUDGE.get(name)
        if judge:
            same = got.returncode == 0 and judge(data, want) is not None and judge(data, got.stdout) == judge(data, want)
        else:
            same = got.returncode == 0 and got.stdout.split() == want.split()
        if not same:
            return f"MISMATCH on input {i}:\n{data}want {' '.join(want.split())}\ngot  {' '.join(got.stdout.split())} {got.stderr[-400:]}"
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
