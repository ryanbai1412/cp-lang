#!/usr/bin/env python3
"""Differential harness for blind_critic_1 ports against corpus/submissions/ originals."""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def build_pair(stem, tmp):
    dst = os.path.join(tmp, f"{stem}.cp")
    shutil.copy(os.path.join(HERE, f"{stem}.cp"), dst)
    r = subprocess.run([sys.executable, CP, "build", dst], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"build {stem} failed: {r.stderr[-400:]}")
    cp_bin = os.path.join(tmp, f"{stem}_cp")
    orig_bin = os.path.join(tmp, f"{stem}_orig")
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", cp_bin, dst[:-3] + ".cpp"], check=True)
    orig = os.path.join(ROOT, "corpus", "submissions", f"{stem}.cpp")
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", orig_bin, orig], check=True)
    return cp_bin, orig_bin


def check_4331904(tmp, cases=60):
    cp_bin, orig_bin = build_pair("4331904", tmp)
    rng = random.Random(4331904)
    alpha = "abc"
    for tc in range(cases):
        n = rng.randint(3, 12)
        words = [(rng.choice("abcdefghij"), ch) for ch in alpha]
        while len(words) < n:
            w = "".join(rng.choice(alpha) for _ in range(rng.randint(1, 4)))
            words.append((rng.choice("abcdefghij"), w))
        m = rng.randint(1, 24)
        s = "".join(rng.choice(alpha) for _ in range(m))
        inp = f"{len(words)} {m}\n{s}\n" + "\n".join(f"{c} {w}" for c, w in words) + "\n"
        a = subprocess.run([cp_bin], input=inp, capture_output=True, text=True)
        b = subprocess.run([orig_bin], input=inp, capture_output=True, text=True)
        if a.returncode != 0 or b.returncode != 0 or a.stdout.strip() != b.stdout.strip():
            print(f"FAIL 4331904 case {tc}: cp=({a.returncode},{a.stdout.strip()!r}) orig=({b.returncode},{b.stdout.strip()!r})")
            return False
    print(f"ok   4331904: {cases} cases agree")
    return True


def main():
    tmp = tempfile.mkdtemp(prefix="blind_critic_1_")
    try:
        ok = check_4331904(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
