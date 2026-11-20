#!/usr/bin/env python3
"""Differential check for scout batch 40: each today/<problem>.cp against the
original corpus C++ on random small and medium inputs.

    python3 proto/ports/scout_batch_40/check.py [problem ...] [--cases N]

The ports are built with ASan + UBSan, the originals with plain -O1. Outputs
are compared token by token, and where a brute force is cheap both sides are
also checked against it (cheerio1s2: sort all n*n seat prices; tle18p5: the
printed sequence must be a strictly increasing subsequence of maximum length).

Two originals do not read stdin:
  tle17c5p1  interactive; the port and the original each talk to a judge that
             answers "guess g" with secret - g and must hear "answer secret".
  tle18p5    the judge's header defines the constant N, get_item and
             print_num, and the original recurses on N in templates. Each case
             compiles the original against a generated header, so this one
             runs fewer, smaller cases.
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ORIG = {"cheerio1s1": "5337471", "cheerio1s2": "5337643", "cheerio1s5": "5339756",
        "tle17c5p1": "5034414", "tle18p5": "5876770"}
CASES = {"tle18p5": 40}

def rows(*ls): return "\n".join(" ".join(map(str, l)) for l in ls) + "\n"

def gen_cheerio1s1(r, big):
    n = r.randint(1, 2000 if big else 6)
    k = r.randint(n + 1, 10**9 if big else 40)
    top = r.choice([3, 10**9]) if big else 6
    return rows([n, k], *[[p, r.randint(1, top), r.randint(1, top)] for p in sorted(r.sample(range(1, k), n))])

def gen_cheerio1s2(r, big):
    hi = 10**4 if big else 6
    return rows([r.randint(1, 3000 if big else 8), r.randint(1, hi), r.randint(1, hi)])

def gen_cheerio1s5(r, big):
    n = r.randint(1, 70 if big else 8)
    s = "".join(r.choice("HT" if r.random() < 0.5 else "HTTT") for _ in range(n))
    return f"{s}\n{r.randint(0, n)}\n"

def gen_tle17c5p1(r, big):
    return f"{r.randint(-10**9, 10**9) if big else r.randint(-60, 60)}\n"

def gen_tle18p5(r, big):
    n = r.randint(0, 60 if big else 7)
    top = r.choice([3, n + 1, 10**9])
    return rows([n], [r.randint(-top, top) for _ in range(n)])

def brute_cheerio1s2(inp, out):
    n, a, b = map(int, inp.split())
    if n > 300: return True
    return int(out[0]) == sum(sorted(a * i + b * j for i in range(1, n + 1) for j in range(1, n + 1))[:n])

def brute_tle18p5(inp, out):
    n, *a = map(int, inp.split())
    out = [int(x) for x in out]
    best = [1] * n
    for i in range(n):
        for j in range(i):
            if a[j] < a[i]: best[i] = max(best[i], best[j] + 1)
    seq, at = out[1:], 0
    for x in seq:                       # seq must be a subsequence of a
        while at < n and a[at] != x: at += 1
        if at == n: return False
        at += 1
    return out[0] == len(seq) == max(best, default=0) and all(x < y for x, y in zip(seq, seq[1:]))

def sh(cmd, **kw): return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)

def header(inp):
    """The judge side of tle18p5 for one input: N, get_item, print_num."""
    n, *a = map(int, inp.split())
    return (f"#include <cstdio>\nconstexpr int N = {n};\n"
            f"constexpr long long cp_items[N + 1] = {{{', '.join(map(str, a + [0]))}}};\n"
            "constexpr long long get_item(int i) { return cp_items[i]; }\n"
            'void print_num(long long x) { printf("%lld\\n", x); }\n')

def build(name, tmp):
    cp = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), cp)   # cp.py build rewrites its input
    r = sh(f"python3 {ROOT}/proto/cp.py build {cp}")
    if r.returncode: sys.exit(f"{name}: cp build failed\n{r.stdout}{r.stderr}")
    flags = "-std=c++17 -O1 -g -w -fsanitize=address,undefined -fno-sanitize-recover=all"
    r = sh(f"g++ {flags} -o {tmp}/{name}_cp {tmp}/{name}.cpp")
    if r.returncode: sys.exit(f"{name}: port g++ failed\n{r.stderr[:3000]}")
    if name != "tle18p5": build_orig(name, tmp, "")

def build_orig(name, tmp, prelude):
    open(f"{tmp}/{name}_orig.cpp", "w").write(prelude + open(f"{ROOT}/corpus/submissions/{ORIG[name]}.cpp").read())
    r = sh(f"g++ -std=c++17 -O1 -w -ftemplate-depth=5000 -o {tmp}/{name}_orig {tmp}/{name}_orig.cpp")
    if r.returncode: sys.exit(f"{name}: original g++ failed\n{r.stderr[:3000]}")

def talk(exe, secret, env):
    """One tle17c5p1 conversation; true when the solution names the secret after one guess."""
    p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1, env=env)
    try:
        word, g = p.stdout.readline().split()
        if word != "guess" or not -10**9 <= int(g) <= 10**9: return False
        p.stdin.write(f"{secret - int(g)}\n"); p.stdin.flush()
        return p.stdout.readline().split() == ["answer", str(secret)]
    except (ValueError, BrokenPipeError):
        return False
    finally:
        p.kill(); p.wait()

def main():
    args = sys.argv[1:]
    cases = None
    if "--cases" in args:
        i = args.index("--cases"); cases = int(args[i + 1]); del args[i:i + 2]
    env = dict(os.environ, ASAN_OPTIONS="detect_leaks=0:detect_stack_use_after_return=0")
    with tempfile.TemporaryDirectory() as tmp:
        for name in args or list(ORIG):
            build(name, tmp)
            total = cases or CASES.get(name, 300)
            rng = random.Random(40_000 + sum(map(ord, name)))
            gen, brute = globals()["gen_" + name], globals().get("brute_" + name)
            for case in range(total):
                big = case >= total * 2 // 3          # last third: medium inputs
                inp = gen(rng, big)
                def fail(msg):
                    open(f"/tmp/{name}_fail.in", "w").write(inp)
                    sys.exit(f"{name} case {case}: {msg} (input in /tmp/{name}_fail.in)")
                if name == "tle17c5p1":
                    for side in ("cp", "orig"):
                        if not talk(f"{tmp}/{name}_{side}", int(inp), env): fail(f"{side} lost the conversation")
                    continue
                if name == "tle18p5": build_orig(name, tmp, header(inp))
                outs = []
                for side in ("cp", "orig"):
                    r = subprocess.run(f"{tmp}/{name}_{side}", input=inp, capture_output=True, text=True, env=env, timeout=120)
                    if r.returncode: fail(f"{side} exited {r.returncode}\n{r.stderr[-2000:]}")
                    outs.append(r.stdout.split())
                if outs[0] != outs[1]:
                    fail(f"outputs differ\nport: {' '.join(outs[0])[:400]}\norig: {' '.join(outs[1])[:400]}")
                if brute and not brute(inp, outs[0]): fail("both sides disagree with the brute force")
            print(f"{name}: {total} cases agree ({total - total * 2 // 3} medium)"
                  + (", brute force agrees" if brute else ""), flush=True)

if __name__ == "__main__":
    main()
