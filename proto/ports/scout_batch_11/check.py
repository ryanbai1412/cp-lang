#!/usr/bin/env python3
"""Check the batch-11 scout ports against the original submissions.

A port with a generator below runs on the reference interpreter
(`proto/cp.py lib-run`) with small random inputs: it imports `std.x` or
uses tables that `proto/cpc` does not build. Every other port is built by `proto/cpc` and stressed with inputs generated from
its own input types (`proto/cp.py stress`). Both sides are compared with the
original named on the port's `// original:` line.

    python3 proto/ports/scout_batch_11/check.py            # every port, 200 inputs each
    python3 proto/ports/scout_batch_11/check.py noi18p4 50     # one port, 50 inputs
"""
import glob, os, random, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_noi18p4():
    rows = [(random.randint(1, 4),)]
    for _ in range(rows[0][0]):
        n, m = random.randint(1, 5), random.randint(1, 5)
        rows += [(n, m), [random.randint(1, 40) for _ in range(n)], [random.choice([1, 2, 3, 4, 5, 6, 8, 9, 10, 12]) for _ in range(n)],
                 [random.randint(1, 12) for _ in range(n)], [random.randint(1, 12) for _ in range(m)]]
    return lines(*rows)


def gen_noi21p2():
    rows = [(random.randint(1, 3),)]
    for _ in range(rows[0][0]):
        k, ends = random.randint(2, 4), random.randint(1, 3)
        size = [ends] + [random.randint(1, 4) for _ in range(k - 2)] + [ends]
        gaps = [random.sample([(u, v) for u in range(1, size[i] + 1) for v in range(1, size[i + 1] + 1)],
                              random.randint(1, size[i] * size[i + 1])) for i in range(k - 1)]
        rows += [(k,), size, [len(g) for g in gaps]] + [e for g in gaps for e in g]
    return lines(*rows)


def gen_canada21p5():
    n = random.randint(1, 6)
    bits = lambda: "".join(random.choice("01") for _ in range(n))
    return lines((n, random.randint(1, n), random.choice([random.randint(1, 6), random.randint(1, 10**9)])), (bits(),), (bits(),))


def gen_noi20p1():
    n, m, t = random.randint(1, 3), random.randint(1, 6), random.randint(1, 40)
    days = random.sample(range(1, t + 1), random.randint(0, min(3, t)))
    return lines((n, m, t, len(days)), [random.randint(1, 10) for _ in range(n)],
                 *[(random.randint(1, n), random.randint(1, n), random.randint(1, 5)) for _ in range(m)],
                 *[(d, random.randint(1, n), random.randint(1, 50)) for d in days])


def gen_noi07p5():
    k = random.randint(2, 3)
    return lines((k, random.choice([random.randint(k, 30), random.randint(k, 10**15)])))


def gen_sdoi17r1p1():
    t = random.randint(1, 5)
    return lines((t,), *[(random.randint(1, 60), random.randint(1, 60)) for _ in range(t)])


def gen_shoi16p3():
    m, n = random.randint(2, 6), random.randint(1, 4)
    return lines((m, n, random.randint(0, m - 1)), [random.choice([random.randint(1, 8), random.randint(1, 10**9)]) for _ in range(n)],
                 [random.randint(1, m) for _ in range(n)])


GENS = {"noi18p4": gen_noi18p4, "noi21p2": gen_noi21p2, "canada21p5": gen_canada21p5, "noi20p1": gen_noi20p1,
        "noi07p5": gen_noi07p5, "sdoi17r1p1": gen_sdoi17r1p1, "shoi16p3": gen_shoi16p3}
# Ports that are slow on the interpreter or sieve to 1e7 on every run get fewer inputs.
AT_MOST = {"noi07p5": 40, "noi20p1": 60, "noip21p1": 20}


def run(cmd, text):
    return subprocess.run(cmd, input=text, capture_output=True, text=True, cwd=ROOT)


def check(name, iters, tmp):
    iters = min(iters, AT_MOST.get(name, iters))
    port = os.path.join(HERE, name + ".cp")
    src = open(port).read()
    orig = os.path.join(ROOT, re.search(r"^// original: (\S+)", src, re.M).group(1))
    if name not in GENS:
        copy = os.path.join(tmp, name + ".cp")      # built from a copy: the build stamps its source
        shutil.copy(port, copy)
        out = subprocess.run(["bash", "-c", f'ulimit -s unlimited; python3 cp.py stress "{copy}" "{orig}" {iters} 5'],
                             capture_output=True, text=True, cwd=os.path.join(ROOT, "proto"))
        last = (out.stdout + out.stderr).strip().splitlines()[-1]
        return last.startswith("stress OK"), last
    exe = os.path.join(tmp, name)
    subprocess.run(["g++", "-std=c++17", "-O2", "-w", "-o", exe, orig], check=True)
    for k in range(iters):
        text = GENS[name]()
        want, got = run([exe], text), run([sys.executable, CP, "lib-run", port], text)
        if want.stdout.split() != got.stdout.split():
            return False, f"differs on input {k}:\n{text}original: {want.stdout}port: {got.stdout}{got.stderr}"
    return True, f"interpreter OK: {iters} tests agree"


def main():
    names = [a for a in sys.argv[1:] if not a.isdigit()] or sorted(os.path.basename(p)[:-3] for p in glob.glob(HERE + "/*.cp"))
    iters = next((int(a) for a in sys.argv[1:] if a.isdigit()), 200)
    random.seed(11)
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            ok, note = check(name, iters, tmp)
            print(f"{name}: {note}")
            bad += not ok
    sys.exit(1 if bad else 0)


main()
