#!/usr/bin/env python3
"""Script-made inputs for the two today/ forms that `cp.py stress` cannot check.

btoi17p1 needs symmetric lists, which a `gen:` function writes without their lengths (K4);
noi06p4 uses `std.flow`, so it only runs on the reference interpreter (F1).
Called by check.sh: feed.py <tmp dir> <iterations>."""
import os, random, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def original(name):
    for line in open(os.path.join(HERE, "today", name + ".cp")):
        if line.startswith("// original: "):
            return os.path.join(ROOT, line.split()[2])


def gen_btoi17p1(r):
    n = r.randint(1, 14)
    adj = [[] for _ in range(n)]
    p = r.choice([0.2, 0.5, 0.8])
    for a in range(n):
        for b in range(a + 1, n):
            if r.random() < p and len(adj[a]) < 9 and len(adj[b]) < 9:
                adj[a].append(b), adj[b].append(a)
    return f"{n} 10\n" + "".join(" ".join(map(str, [len(x)] + x)) + "\n" for x in adj)


def gen_noi06p4(r):
    n, m = r.randint(1, 7), r.randint(1, 8)
    costs = " ".join(str(r.randint(1, 12)) for _ in range(n))
    users = "".join(f"{r.randint(1, n)} {r.randint(1, n)} {r.randint(1, 12)}\n" for _ in range(m))
    return f"{n} {m}\n{costs}\n{users}"


def main():
    tmp, iters = sys.argv[1], int(sys.argv[2])
    port = os.path.join(tmp, "btoi17p1.cp")
    open(port, "w").write(open(os.path.join(HERE, "today", "btoi17p1.cp")).read())
    subprocess.run([sys.executable, CP, "build", port], check=True, capture_output=True)
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", port[:-3], port[:-3] + ".cpp"], check=True)
    runs = {"btoi17p1": [port[:-3]],
            "noi06p4": [sys.executable, CP, "lib-run", os.path.join(HERE, "today", "noi06p4.cp")]}
    for name, ours in runs.items():
        ref = os.path.join(tmp, name + "_ref")
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ref, original(name)], check=True)
        for seed in range(iters):
            inp = globals()["gen_" + name](random.Random(seed))
            a = subprocess.run(ours, input=inp, capture_output=True, text=True)
            b = subprocess.run([ref], input=inp, capture_output=True, text=True)
            if a.returncode or a.stdout.split() != b.stdout.split():
                sys.exit(f"MISMATCH {name} seed={seed}\ninput:\n{inp}\nours:\n{a.stdout}{a.stderr[-400:]}\nref:\n{b.stdout}")
    print(f"{iters} script-made inputs each agree")


main()
