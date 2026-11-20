#!/usr/bin/env python3
"""Differential check for scout batch 35: each port against its accepted C++.

    python3 proto/ports/scout_batch_35/check.py              # every port, 200 inputs each
    python3 proto/ports/scout_batch_35/check.py coci21c1p4 500

The top-level `<code>.cp` files are the ports as the v2 cheat sheet spells
them; neither prototype accepts all of v2, so what runs is `today/<code>.cp`,
the same algorithm in the forms the prototype takes. It is built to C++17
by `proto/cp.py build` (on a temporary copy, because the build stamps its
input). coi07p1 needs floating point and coci21c2p1 string interpolation,
which only the reference interpreter has, so those two run under
`proto/cp.py lib-run` on SLOW small inputs. About one
built input in ten is medium-sized.
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")
SLOW = 60
INTERPRETED = {"coi07p1", "coci21c2p1"}


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) if isinstance(r, (list, tuple)) else str(r) for r in rows) + "\n"


def word(rng, n=None):
    return "".join(rng.choice("abcdefgh") for _ in range(n or rng.randint(1, 6)))


def gen_kaucuk(rng, big):
    n = rng.randint(1, 100 if big else 8)
    rows, depth = [], 0
    for _ in range(n):
        depth = rng.randint(1, min(3, depth + 1))
        rows.append(("sub" * (depth - 1) + "section", word(rng)))
    return lines(n, *rows)


def gen_ljeto(rng, big):
    n = rng.randint(1, 100 if big else 8)
    t, rows = 0, []
    for _ in range(n):
        t += rng.choice([1, 5, 9, 10, 11, 30])
        a = rng.randint(1, 8)
        rows.append((t, a, rng.randint(5, 8) if a <= 4 else rng.randint(1, 4)))
    return lines(n, *rows)


def gen_kutije(rng, big):
    n, m, q = (rng.randint(1, 300), rng.randint(1, 20), 400) if big else (rng.randint(1, 7), rng.randint(1, 3), 10)
    perms = []
    for _ in range(m):
        p = list(range(1, n + 1))
        for _ in range(rng.randint(0, 2 if not big else n // 3)):   # few swaps, so not everything joins
            i, j = rng.randrange(n), rng.randrange(n)
            p[i], p[j] = p[j], p[i]
        perms.append(p)
    return lines((n, m, q), *perms, *[(rng.randint(1, n), rng.randint(1, n)) for _ in range(q)])


def gen_lista(rng, big):
    n = rng.randint(1, 100 if big else 12)
    k = rng.randint(2, 4)
    a = [rng.randint(1, 2) for _ in range(n)]
    if rng.random() < 0.5 and n >= k:                               # plant a repeat
        length = rng.randint(1, n // k)
        at = rng.randint(0, n - k * length)
        block = [rng.randint(1, 3) for _ in range(length)]
        a[at:at + k * length] = block * k
    return lines((n, k), a)


def forest(rng, n, keep):
    return [(rng.randint(1, v - 1), v) for v in range(2, n + 1) if rng.random() < keep]


def gen_cijano(rng, big):
    n = rng.randint(1, 2000 if big else 10)
    e = forest(rng, n, rng.choice([0.3, 0.7, 1.0]))
    rng.shuffle(e)
    return lines((n, len(e)), *[rng.sample(x, 2) for x in e])


def gen_med(rng, big):
    n = rng.randint(1, 300 if big else 6)
    names = set()
    while len(names) < n: names.add(word(rng, rng.randint(1, 3)))
    top = rng.choice([1, 3, 150, 500])                              # low tops force ties
    return lines(n, *[[nm] + [rng.randint(0, top) for _ in range(5)] for nm in sorted(names, key=lambda _: rng.random())])


def gen_zemljiste(rng, big):
    n, m = (rng.randint(1, 40), rng.randint(1, 40)) if big else (rng.randint(1, 5), rng.randint(1, 5))
    top = rng.choice([3, 20, 10 ** 9])
    return lines((n, m, rng.randint(1, top * 4), rng.randint(1, top * 4)),
                 *[[rng.randint(1, top) for _ in range(m)] for _ in range(n)])


def gen_coi07(rng, big):
    n = rng.randint(1, 2000 if big else 8)
    k = rng.randint(1, 20) if rng.random() < 0.5 else round(rng.uniform(0.5, 20), 1)
    xs = sorted(rng.randint(0, 10 * n) if rng.random() < 0.7 else round(rng.uniform(0, 10 * n), 1) for _ in range(n))
    return lines((k, n), *xs)


def gen_logicari(rng, big):
    n = rng.randint(3, 3000 if big else 9)
    order = list(range(1, n + 1))
    rng.shuffle(order)
    cyc = rng.randint(3, n if rng.random() < 0.3 else min(n, 6))
    e = [(order[i], order[(i + 1) % cyc]) for i in range(cyc)]
    e += [(order[rng.randrange(i)], order[i]) for i in range(cyc, n)]
    rng.shuffle(e)
    return lines(n, *[rng.sample(x, 2) for x in e])


def gen_set(rng, big):
    k = rng.randint(1, 7 if big else 3)
    n = rng.randint(1, 3 ** k)
    cards = rng.sample(range(3 ** k), n)
    def show(c): return "".join("123"[c // 3 ** i % 3] for i in range(k))
    return lines((n, k), *map(show, cards))


def gen_volont(rng, big):
    n = rng.randint(1, 3000 if big else 9)
    p = list(range(1, n + 1))
    if rng.random() < 0.5:
        rng.shuffle(p)
    else:                                                           # blocks, so several disjoint LIS exist
        b = rng.randint(1, n)
        p = [x for s in range(0, n, b) for x in sorted(p[s:s + b], reverse=rng.random() < 0.5)]
        if rng.random() < 0.5:
            cols = [p[i::b] for i in range(b)]
            p = [x for c in cols for x in c]
    return lines(n, p)


def judge_close(inp, ref, out):
    """Three decimals; a tie in the last place may round either way."""
    return None if abs(float(ref) - float(out)) < 1.5e-3 else "outputs differ"


def judge_volont(inp, ref, out):
    """Any largest family of disjoint longest increasing subsequences is accepted."""
    p = list(map(int, inp.split()))[1:]
    tails = []
    import bisect
    for x in p:
        i = bisect.bisect_left(tails, x)
        tails[i:i + 1] = [x]
    r, o = ref.split("\n"), out.split("\n")
    if r[0] != o[0]: return f"header {o[0]!r}, expected {r[0]!r}"
    count, length = map(int, o[0].split())
    if length != len(tails): return "length is not the LIS length"
    if len(o) != count + 1: return "wrong number of lines"
    used = set()
    for row in o[1:]:
        c = list(map(int, row.split()))
        if len(c) != length: return "a sequence has the wrong length"
        if any(not 1 <= i <= len(p) for i in c) or used & set(c): return "index out of range or reused"
        used |= set(c)
        if any(a >= b or p[a - 1] >= p[b - 1] for a, b in zip(c, c[1:])): return "a sequence is not increasing"
    return None


PROBLEMS = {   # code: (original submission, generator, judge or None)
    "coci21c1p1": (4080824, gen_ljeto, None),
    "coci21c2p1": (4143397, gen_kaucuk, None),
    "coci21c2p2": (4143417, gen_kutije, None),
    "coci21c3p1": (4196130, gen_lista, None),
    "coci21c3p2": (4196157, gen_cijano, None),
    "coci21c6p1": (4529987, gen_med, None),
    "coci21c6p2": (4529998, gen_zemljiste, None),
    "coi07p1": (5323995, gen_coi07, judge_close),
    "coci21c1p3": (4080960, gen_logicari, None),
    "coci21c1p4": (4106618, gen_set, None),
    "coci21c1p5": (4086660, gen_volont, judge_volont),
}


def run(cmd, inp):
    r = subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=120)
    if r.returncode != 0:
        return None, (r.stderr.strip().splitlines() or ["exit %d" % r.returncode])[-1]
    return "\n".join(l.rstrip() for l in r.stdout.strip().split("\n")), None


def gxx(src, exe):
    subprocess.run(["g++", "-std=c++17", "-O1", "-w", "-o", exe, src], check=True)


def check(code, rounds, tmp):
    sub, gen, judge = PROBLEMS[code]
    ref = os.path.join(tmp, code + "_orig")
    gxx(os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp"), ref)
    today = os.path.join(HERE, "today", code + ".cp")
    if code in INTERPRETED:
        forms = [("interpreted", [sys.executable, CP, "lib-run", today], min(rounds, SLOW))]
    else:
        copy = os.path.join(tmp, code + ".cp")
        shutil.copy(today, copy)
        b = subprocess.run([sys.executable, CP, "build", copy], capture_output=True, text=True)
        if b.returncode != 0:
            print(f"{code}: today/ does not build: {(b.stdout + b.stderr).strip().splitlines()[-1]}")
            return False
        gxx(copy + "p", os.path.join(tmp, code))
        forms = [("built", [os.path.join(tmp, code)], rounds)]
    rng = random.Random(f"35/{code}")
    done = {name: 0 for name, _, _ in forms}
    for t in range(rounds):
        inp = gen(rng, code not in INTERPRETED and t % 10 == 9)
        want, err = run([ref], inp)
        if err: print(f"{code}: original failed on input {t}: {err}"); return False
        for name, cmd, limit in forms:
            if t >= limit: continue
            got, err = run(cmd, inp)
            bad = err or (judge(inp, want, got) if judge else (None if got == want else "outputs differ"))
            if bad:
                path = os.path.join(tmp, f"{code}.fail.in")
                open(path, "w").write(inp)
                print(f"{code}: {name} FAILED on input {t}: {bad}\n  input kept at {path}\n  want {want[:200]!r}\n  got  {str(got)[:200]!r}")
                return False
            done[name] += 1
    print(f"{code}: ok ({', '.join(f'{name} {n}' for name, n in done.items())})")
    return True


def main():
    args = sys.argv[1:]
    rounds = int(args.pop()) if args and args[-1].isdigit() else 200
    codes = args or list(PROBLEMS)
    tmp = tempfile.mkdtemp(prefix="batch35_")
    ok = all([check(c, rounds, tmp) for c in codes])
    if ok: shutil.rmtree(tmp)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
