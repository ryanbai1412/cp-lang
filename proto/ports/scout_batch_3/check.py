#!/usr/bin/env python3
"""Differential harness for scout_batch_3/today/ ports against corpus/submissions/ originals."""
import math, os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
TODAY = os.path.join(HERE, "today")
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")
INTERACT = os.path.join(ROOT, "proto", "interact.py")

STRESS_PORTS = [
    ("aac4p3", "4140246"),
    ("ccc03s4", "4182950"),
    ("ccc18s4", "4329704"),
    ("dpr", "4131862"),
    ("modinv", "5034440"),
    ("noip99p2", "5044551"),
]


def build_pair(name, sid, tmp):
    dst = os.path.join(tmp, f"{name}.cp")
    shutil.copy(os.path.join(TODAY, f"{name}.cp"), dst)
    r = subprocess.run([sys.executable, CP, "build", dst], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"build {name} failed: {r.stderr[-400:]}")
    cp_bin = os.path.join(tmp, f"{name}_cp")
    orig_bin = os.path.join(tmp, f"{name}_orig")
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", cp_bin, dst[:-3] + ".cpp"], check=True)
    orig = os.path.join(ROOT, "corpus", "submissions", f"{sid}.cpp")
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", orig_bin, orig], check=True)
    return cp_bin, orig_bin


def check_coci06c1p6(tmp, cases=60):
    cp_bin, orig_bin = build_pair("coci06c1p6", "4183051", tmp)
    rng = random.Random(30616)
    for tc in range(cases):
        r = rng.randint(2, 8)
        c = rng.randint(2, 8)
        grid = ["".join(rng.choice("01") for _ in range(c)) for _ in range(r)]
        inp = f"{r} {c}\n" + "\n".join(grid) + "\n"
        a = subprocess.run([cp_bin], input=inp, capture_output=True, text=True)
        b = subprocess.run([orig_bin], input=inp, capture_output=True, text=True)
        if a.returncode != 0 or b.returncode != 0 or a.stdout.strip() != b.stdout.strip():
            print(f"FAIL coci06c1p6 case {tc}: cp={a.stdout.strip()!r} orig={b.stdout.strip()!r}")
            return False
    print(f"ok   coci06c1p6: {cases} cases agree")
    return True


def check_cco04p4(tmp, cases=60):
    cp_bin, orig_bin = build_pair("cco04p4", "5429989", tmp)
    rng = random.Random(30404)

    def gen_craft():
        sx, sy = rng.randint(-10, 10), rng.randint(-10, 10)
        n = rng.randint(1, 5)
        moves = []
        for _ in range(n):
            d = rng.choice([-4, -3, -2, -1, 1, 2, 3, 4])
            ax = rng.choice(["X", "Y"])
            moves.append(f"{d} {ax}")
        return f"{sx} {sy}\n{n}\n" + "\n".join(moves)

    for tc in range(cases):
        inp = gen_craft() + "\n" + gen_craft() + "\n"
        a = subprocess.run([cp_bin], input=inp, capture_output=True, text=True)
        b = subprocess.run([orig_bin], input=inp, capture_output=True, text=True)
        if a.returncode != 0 or b.returncode != 0:
            print(f"FAIL cco04p4 case {tc}: rc=({a.returncode},{b.returncode})")
            return False
        got = f"{math.sqrt(int(a.stdout.strip())):.2f}"
        exp = b.stdout.strip()
        if got != exp and not math.isclose(math.sqrt(int(a.stdout.strip())), float(exp), abs_tol=0.011):
            print(f"FAIL cco04p4 case {tc}: sqrt({a.stdout.strip()})={got} orig={exp}")
            return False
    print(f"ok   cco04p4: {cases} cases agree")
    return True


def check_cco09p4(tmp, cases=30):
    dst = os.path.join(tmp, "cco09p4.cp")
    shutil.copy(os.path.join(TODAY, "cco09p4.cp"), dst)
    orig = os.path.join(ROOT, "corpus", "submissions", "5034519.cpp")
    judge = os.path.join(TODAY, "cco09p4_judge.py")
    r = subprocess.run(
        [sys.executable, INTERACT, dst, orig, judge, str(cases), "8"],
        capture_output=True, text=True,
    )
    if r.returncode == 0:
        print(f"ok   cco09p4: {r.stdout.strip().splitlines()[-1]}")
        return True
    print(f"FAIL cco09p4: {(r.stdout + r.stderr).strip()[-300:]}")
    return False


def main():
    wanted = set(sys.argv[1:])
    iters = int(os.environ.get("ITERS", "50"))
    tmp = tempfile.mkdtemp(prefix="scout_batch_3_")
    fail = 0
    try:
        for name, sid in STRESS_PORTS:
            if wanted and name not in wanted:
                continue
            dst = os.path.join(tmp, f"{name}.cp")
            shutil.copy(os.path.join(TODAY, f"{name}.cp"), dst)
            orig = os.path.join(ROOT, "corpus", "submissions", f"{sid}.cpp")
            r = subprocess.run(
                [sys.executable, CP, "stress", dst, orig, str(iters), "6"],
                capture_output=True, text=True,
            )
            ok_line = next((l for l in r.stdout.splitlines() if "stress OK" in l), None)
            if r.returncode == 0 and ok_line:
                print(f"ok   {name}: {ok_line.strip()}")
            else:
                print(f"FAIL {name}: rc={r.returncode} {(r.stdout + r.stderr).strip()[-300:]}")
                fail += 1
        if not wanted or "coci06c1p6" in wanted:
            if not check_coci06c1p6(tmp):
                fail += 1
        if not wanted or "cco04p4" in wanted:
            if not check_cco04p4(tmp):
                fail += 1
        if not wanted or "cco09p4" in wanted:
            if not check_cco09p4(tmp):
                fail += 1
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
