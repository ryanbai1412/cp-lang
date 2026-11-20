#!/usr/bin/env python3
"""Differential check for scout batch 41: each today/<problem>.cp against the
original corpus submission (7 C++, 3 Java) on random small and medium inputs.

    python3 proto/ports/scout_batch_41/check.py [problem ...] [--cases N]

Compiled ports (proto/cpc) are built with ASan + UBSan and checked on 300
inputs each (200 small, 100 medium). Two unknown-length Str ports (nccc8s1,
nccc8s2) run under proto/cp.py lib-run on 60 inputs each (40 small, 20 medium).
Outputs are compared line-by-line (preserving empty lines for nccc8s1).
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {
    "oly20practice139": ("6415690.cpp", None),
    "oly23practice5":   ("6415646.cpp", None),
    "nccc9s4":          ("4145494.cpp", None),
    "nccc9s1":          ("4051682.cpp", None),
    "nccc9s2":          ("4051671.cpp", None),
    "nccc9s3":          ("4051697.cpp", None),
    "nccc10s1":         ("4214147.cpp", None),
    "nccc8s1":          ("3381111.java", "Problem1"),
    "nccc8s2":          ("3381135.java", "Problem2"),
    "nccc8s4":          ("3381215.java", "Problem4"),
}
INTERPRETED = {"nccc8s1", "nccc8s2"}


def rows(*ls):
    return "\n".join(" ".join(map(str, l)) if isinstance(l, (list, tuple)) else str(l) for l in ls) + "\n"


def gen_oly20practice139(r, big):
    if big:
        n = r.randint(450, 1400)
        hi = r.choice([50, 1000, 10**9])
    else:
        n = r.randint(1, 16)
        hi = r.choice([5, 25, 1000])
    pts = [(r.randint(0, hi), r.randint(0, hi)) for _ in range(n)]
    return rows([n], *pts)


def gen_oly23practice5(r, big):
    t = r.randint(1, 3 if big else 5)
    out = [[t]]
    for _ in range(t):
        n = r.randint(80, 450) if big else r.randint(1, 14)
        alpha = r.choice([2, 3, 4, n])
        alpha = max(1, min(alpha, n))
        arr = [r.randint(1, alpha) for _ in range(n)]
        out.append([n])
        out.append(arr)
    return rows(*out)


def gen_nccc9s4(r, big):
    if big and r.random() < 0.6:
        # Force > 800 distinct values to exercise the Rollback Mo branch
        n = r.randint(900, 1400)
        q = r.randint(80, 220)
        # Ensure at least 810 distinct values in 1..n
        base = list(range(1, 821))
        rest = [r.randint(1, n) for _ in range(n - len(base))]
        arr = base + rest
        r.shuffle(arr)
    elif big:
        n = r.randint(60, 400)
        q = r.randint(50, 200)
        alpha = r.choice([3, 15, min(n, 200)])
        arr = [r.randint(1, alpha) for _ in range(n)]
    else:
        n = r.randint(1, 30)
        q = r.randint(1, 30)
        alpha = r.choice([1, 2, 5, n])
        arr = [r.randint(1, min(alpha, n)) for _ in range(n)]
    qs = []
    for _ in range(q):
        l = r.randint(1, n)
        rr = r.randint(l, n)
        qs.append((l, rr))
    return rows([n, q], arr, *qs)


def gen_nccc9s1(r, big):
    t = r.randint(5, 25)
    out = [[t]]
    for _ in range(t):
        if big:
            # Keep a + 2b + 3c within 32-bit int for original 4051682.cpp
            out.append([r.randint(0, 200_000_000), r.randint(0, 200_000_000), r.randint(0, 200_000_000)])
        else:
            out.append([r.randint(0, 8), r.randint(0, 8), r.randint(0, 8)])
    return rows(*out)


def gen_nccc9s2(r, big):
    n = r.randint(100, 1500) if big else r.randint(1, 12)
    chars = ["I"] * n + ["U"] * n
    r.shuffle(chars)
    return rows([n], "".join(chars))


def gen_nccc9s3(r, big):
    n = r.randint(100, 1500) if big else r.randint(1, 12)
    chars = ["I"] * n + ["U"] * n
    r.shuffle(chars)
    return rows([n], "".join(chars))


def gen_nccc10s1(r, big):
    if big:
        n = r.randint(40, 250)
        k = r.randint(6, 10)
    else:
        n = r.randint(1, 12)
        k = r.randint(1, 6)
    lines = ["".join(r.choice("TF") for _ in range(k)) for _ in range(n)]
    return rows([n, k], *lines)


def gen_nccc8s1(r, big):
    if big:
        al_k = r.randint(3, 8)
        n = r.randint(6, 12)
    else:
        al_k = r.randint(1, 4)
        n = r.randint(1, 5)
    al = "".join(sorted(r.sample("abcdefghij", al_k)))
    s = "".join(r.choice(al) for _ in range(n))
    return rows(al, s)


def gen_nccc8s2(r, big):
    n = r.randint(120, 450) if big else r.randint(1, 25)
    alpha = "abcdefghijklmnopqrstuvwxyz"[: r.choice([3, 8, 26])]
    s = "".join(r.choice(alpha) for _ in range(n))
    return rows(s)


def gen_nccc8s4(r, big):
    if big:
        m = r.randint(15, 120)
    else:
        m = r.randint(1, 10)
    # Mix sparse acyclic-likely and dense cyclic-likely distributions
    mode = r.randint(0, 2)
    tiles = []
    if mode == 0:
        # Acyclic-leaning: small number of tiles, mostly zeros and positive labels
        for _ in range(m):
            tiles.append([r.choice([0, 0, 0, r.randint(1, 26)]) for _ in range(4)])
    elif mode == 1:
        # Small label pool -> often cyclic
        pool = [0, 0] + list(range(-5, 6))
        for _ in range(m):
            tiles.append([r.choice(pool) for _ in range(4)])
    else:
        # Full -26..26 with ~40% zeros
        for _ in range(m):
            tiles.append([0 if r.random() < 0.4 else r.choice([r.randint(-26, -1), r.randint(1, 26)]) for _ in range(4)])
    return rows([m], *tiles)


def sh(cmd, **kw):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)


def build(name, tmp):
    orig_file, java_cls = ORIG[name]
    if name not in INTERPRETED:
        cp = os.path.join(tmp, name + ".cp")
        shutil.copy(os.path.join(HERE, "today", name + ".cp"), cp)
        r = sh(f"python3 {ROOT}/proto/cp.py build {cp}")
        if r.returncode:
            sys.exit(f"{name}: cp build failed\n{r.stdout}{r.stderr}")
        flags = "-std=c++17 -O1 -g -w -fsanitize=address,undefined -fno-sanitize-recover=all"
        r = sh(f"g++ {flags} -o {tmp}/{name}_cp {tmp}/{name}.cpp")
        if r.returncode:
            sys.exit(f"{name}: port g++ failed\n{r.stderr[:3000]}")
    if java_cls is None:
        r = sh(f"g++ -std=c++17 -O1 -w -o {tmp}/{name}_orig {ROOT}/corpus/submissions/{orig_file}")
        if r.returncode:
            sys.exit(f"{name}: original g++ failed\n{r.stderr[:3000]}")
    else:
        jdir = os.path.join(tmp, f"java_{name}")
        os.makedirs(jdir, exist_ok=True)
        shutil.copy(f"{ROOT}/corpus/submissions/{orig_file}", os.path.join(jdir, f"{java_cls}.java"))
        r = sh(f"javac {jdir}/{java_cls}.java")
        if r.returncode:
            sys.exit(f"{name}: original javac failed\n{r.stderr[:3000]}")


def run_side(name, side, tmp, inp, env):
    orig_file, java_cls = ORIG[name]
    if side == "cp":
        if name in INTERPRETED:
            cmd = f"python3 {ROOT}/proto/cp.py lib-run {HERE}/today/{name}.cp"
        else:
            cmd = f"ulimit -s 1048576; {tmp}/{name}_cp"
    else:
        if java_cls is None:
            cmd = f"ulimit -s 1048576; {tmp}/{name}_orig"
        else:
            cmd = f"java -cp {tmp}/java_{name} {java_cls}"
    return subprocess.run(cmd, shell=True, input=inp, capture_output=True, text=True, env=env, timeout=120)


def normalize(name, text):
    # Preserve blank lines for nccc8s1 (deleting the only char yields "")
    if name == "nccc8s1":
        return [line.rstrip("\r") for line in text.splitlines()]
    return text.split()


def main():
    args = sys.argv[1:]
    cases_override = None
    if "--cases" in args:
        i = args.index("--cases")
        cases_override = int(args[i + 1])
        del args[i : i + 2]
    names = args or list(ORIG)
    env = dict(os.environ, ASAN_OPTIONS="detect_leaks=0:detect_stack_use_after_return=0")
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            build(name, tmp)
            cases = cases_override if cases_override is not None else (60 if name in INTERPRETED else 300)
            rng = random.Random(41_000 + sum(map(ord, name)))
            gen = globals()["gen_" + name]
            for case in range(cases):
                big = case >= cases * 2 // 3
                inp = gen(rng, big)
                outs = []
                for side in ("cp", "orig"):
                    r = run_side(name, side, tmp, inp, env)
                    if r.returncode:
                        open(f"/tmp/{name}_fail.in", "w").write(inp)
                        sys.exit(f"{name} case {case}: {side} exited {r.returncode} (input in /tmp/{name}_fail.in)\n{r.stderr[-2000:]}")
                    outs.append(normalize(name, r.stdout))
                if outs[0] != outs[1]:
                    open(f"/tmp/{name}_fail.in", "w").write(inp)
                    sys.exit(
                        f"{name} case {case}: outputs differ (input in /tmp/{name}_fail.in)\n"
                        f"port: {outs[0][:20]}\norig: {outs[1][:20]}"
                    )
            mode = "lib-run" if name in INTERPRETED else "cpc+ASan/UBSan"
            print(f"{name:18s}: {cases:3d} cases agree ({cases - cases * 2 // 3:3d} medium) [{mode}]", flush=True)


if __name__ == "__main__":
    main()
