#!/usr/bin/env python3
"""Differential check for the 8 buildable ports in scout_batch_14/today/.

Builds each .cp file from a temporary copy (so tracked .cp files stay untouched),
compiles the emitted C++17 and the original corpus submission with g++ -O2, and
compares stdout across deterministic + random test cases.
"""
import os
import random
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
TODAY = os.path.dirname(os.path.abspath(__file__))

PROBLEMS = [
    ("noip21p2", "corpus/submissions/5445677.cpp"),
    ("csps21p2", "corpus/submissions/4819734.cpp"),
    ("dmopc18c1p6", "corpus/submissions/4155900.cpp"),
    ("noip00p4", "corpus/submissions/5067920.cpp"),
    ("noi15p3", "corpus/submissions/3782724.cpp"),
    ("noi23p2", "corpus/submissions/5723059.cpp"),
    ("ioi16p6io", "corpus/submissions/4730558.cpp"),
    ("noi19p1", "corpus/submissions/3790564.cpp"),
]


def gen_cases(name: str, rng: random.Random) -> list[str]:
    cases: list[str] = []
    if name == "noip21p2":
        for _ in range(30):
            n = rng.randint(1, 8)
            m = rng.randint(0, 6)
            k = rng.randint(1, n)
            v = [rng.randint(0, 998244352) for _ in range(m + 1)]
            cases.append(f"{n} {m} {k}\n" + " ".join(map(str, v)) + "\n")
    elif name == "csps21p2":
        cases.append("7 3\n(*??*?)\n")
        cases.append("10 2\n???(*??(?)\n")
        for _ in range(30):
            n = rng.randint(2, 16)
            k = rng.randint(1, n)
            s = "".join(rng.choice("()*????") for _ in range(n))
            cases.append(f"{n} {k}\n{s}\n")
    elif name == "dmopc18c1p6":
        cases.append("1 3\n101\n")
        for _ in range(30):
            n = rng.randint(1, 15)
            m = rng.randint(1, 10)
            rows = ["".join(rng.choice("01") for _ in range(m)) for _ in range(n)]
            cases.append(f"{n} {m}\n" + "\n".join(rows) + "\n")
    elif name == "noip00p4":
        cases.append("8\n2 3 13\n2 6 6\n3 5 7\n4 4 14\n5 2 21\n5 6 4\n6 3 15\n7 2 14\n0 0 0\n")
        for _ in range(25):
            n = rng.randint(2, 8)
            cnt = rng.randint(1, min(12, n * n))
            pts = rng.sample([(r, c) for r in range(1, n + 1) for c in range(1, n + 1)], cnt)
            lines = [str(n)] + [f"{r} {c} {rng.randint(1, 50)}" for r, c in pts] + ["0 0 0"]
            cases.append("\n".join(lines) + "\n")
    elif name == "noi15p3":
        cases.extend(["3 10000\n", "4 10000\n", "10 10000\n"])
        for n in [2, 5, 8, 12, 20, 30, 50, 100, 200]:
            p = rng.choice([7, 97, 10007, 998244353, 1000000000])
            cases.append(f"{n} {p}\n")
    elif name == "noi23p2":
        for _ in range(20):
            t = rng.randint(1, 3)
            lines = [f"0 {t}"]
            for _ in range(t):
                n = rng.randint(1, 10)
                m = rng.randint(0, 8)
                k = rng.randint(0, 6)
                pars = [rng.randint(1, i) for i in range(1, n)]
                lines.append(f"{n} {m} {k}")
                lines.append(" ".join(map(str, pars)))
            cases.append("\n".join(lines) + "\n")
    elif name == "ioi16p6io":
        cases.append("5 7 2\n0 3\n4 4\n4 6\n4 5\n4 6\n")
        for _ in range(30):
            m = rng.randint(2, 40)
            n = rng.randint(1, 15)
            k = rng.randint(1, n)
            pts = [(rng.randint(0, m - 1), rng.randint(0, m - 1)) for _ in range(n)]
            lines = [f"{n} {m} {k}"] + [f"{r} {c}" for r, c in pts]
            cases.append("\n".join(lines) + "\n")
    elif name == "noi19p1":
        cases.append("2 2 1 10 10\n1 2 1 3\n1 2 2 5\n")
        for _ in range(30):
            n = rng.randint(2, 8)
            m = rng.randint(n, 25)
            A = rng.randint(0, 5)
            B = rng.randint(0, 20)
            C = rng.randint(0, 20)
            routes = []
            t = 0
            for u in range(1, n):
                s = t + rng.randint(0, 5)
                e = s + rng.randint(1, 5)
                routes.append((u, u + 1, s, e))
                t = e
            while len(routes) < m:
                u = rng.randint(1, n - 1)
                v = rng.randint(u + 1, n)
                s = rng.randint(0, max(1, t))
                e = s + rng.randint(1, 10)
                routes.append((u, v, s, e))
            rng.shuffle(routes)
            lines = [f"{n} {len(routes)} {A} {B} {C}"] + [f"{u} {v} {s} {e}" for u, v, s, e in routes]
            cases.append("\n".join(lines) + "\n")
    return cases


def main() -> int:
    rng = random.Random(1412)
    tmp = tempfile.mkdtemp(prefix="cp_b14_stress_")
    try:
        for name, orig_rel in PROBLEMS:
            cp_dst = os.path.join(tmp, f"{name}.cp")
            shutil.copy(os.path.join(TODAY, f"{name}.cp"), cp_dst)
            r = subprocess.run(
                [sys.executable, os.path.join(REPO, "proto", "cp.py"), "build", cp_dst],
                capture_output=True,
                text=True,
            )
            if r.returncode != 0:
                print(f"FAIL build {name}: {r.stderr.strip()}")
                return 1
            cp_bin = os.path.join(tmp, f"{name}_cp")
            orig_bin = os.path.join(tmp, f"{name}_orig")
            g1 = subprocess.run(["g++", "-O2", "-std=c++17", cp_dst[:-3] + ".cpp", "-o", cp_bin], capture_output=True, text=True)
            g2 = subprocess.run(["g++", "-O2", "-std=c++17", os.path.join(REPO, orig_rel), "-o", orig_bin], capture_output=True, text=True)
            if g1.returncode != 0 or g2.returncode != 0:
                print(f"FAIL g++ {name}: cp={g1.returncode} orig={g2.returncode}\n{g1.stderr}\n{g2.stderr}")
                return 1
            cases = gen_cases(name, rng)
            for idx, inp in enumerate(cases):
                o1 = subprocess.run([cp_bin], input=inp, capture_output=True, text=True, timeout=5)
                o2 = subprocess.run([orig_bin], input=inp, capture_output=True, text=True, timeout=5)
                if o1.returncode != 0 or o2.returncode != 0 or o1.stdout.strip() != o2.stdout.strip():
                    print(f"MISMATCH {name} case #{idx}: cp={o1.stdout.strip()!r} orig={o2.stdout.strip()!r}")
                    return 1
            print(f"PASS {name}: {len(cases)}/{len(cases)} cases")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
