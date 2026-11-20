#!/usr/bin/env python3
"""Check the batch-8 scout ports against the original submissions.

A port that imports `std.x` runs on the reference interpreter
(`proto/cp.py lib-run`) with the small random inputs generated below. Every
other port is built by `proto/cpc` and stressed with inputs generated from
its own input types (`proto/cp.py stress`). Both sides are compared with the
original named on the port's `// original:` line.

    python3 proto/ports/scout_batch_8/check.py            # every port, 200 inputs each
    python3 proto/ports/scout_batch_8/check.py dpy 50     # one port, 50 inputs
"""
import glob, os, random, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_dpy():
    h, w = random.randint(2, 6), random.randint(2, 6)
    cells = [(r, c) for r in range(1, h + 1) for c in range(1, w + 1) if (r, c) not in ((1, 1), (h, w))]
    walls = random.sample(cells, random.randint(1, min(len(cells), 6)))
    return lines((h, w, len(walls)), *walls)


def gen_ioi04p5():
    n, m = random.randint(0, 5), random.randint(0, 5)
    fields = [random.randint(3, 9) for _ in range(n)]
    strips = [random.randint(2, 9) for _ in range(m)]
    q = random.randint(1, max(1, sum(fields) + sum(strips)))
    return lines((q, n, m), fields, strips)


def gen_yac1p4():
    n = random.randint(1, 7)
    return lines((n,), ("".join(random.choice("ABCDZ") for _ in range(n)),))


GENS = {"dpy": gen_dpy, "ioi04p5": gen_ioi04p5, "yac1p4": gen_yac1p4}


def run(cmd, text):
    return subprocess.run(cmd, input=text, capture_output=True, text=True, cwd=ROOT)


def check(name, iters, tmp):
    port = os.path.join(HERE, name + ".cp")
    src = open(port).read()
    orig = os.path.join(ROOT, re.search(r"^// original: (\S+)", src, re.M).group(1))
    if "\nuse std." not in src:
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
    random.seed(8)
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            ok, note = check(name, iters, tmp)
            print(f"{name}: {note}")
            bad += not ok
    sys.exit(1 if bad else 0)


main()
