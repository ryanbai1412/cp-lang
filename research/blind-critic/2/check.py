#!/usr/bin/env python3
"""Differential harness for blind_critic_2 ports against corpus/submissions/ originals."""
import os, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")

PORTS = [
    ("dpq", "4131861"),
    ("noi15p4", "5032756"),
    ("sac22cc2p4", "4121086"),
    ("ccc02s4", "4157526"),
    ("cco07p2", "3742713"),
    ("sac22cc1p5", "3992625"),
    ("dpz", "4133674"),
    ("dmpg16s5", "5323572"),
]


def main():
    wanted = set(sys.argv[1:])
    iters = int(os.environ.get("ITERS", "50"))
    tmp = tempfile.mkdtemp(prefix="blind_critic_2_")
    fail = 0
    try:
        for name, sid in PORTS:
            if wanted and name not in wanted:
                continue
            dst = os.path.join(tmp, f"{name}.cp")
            shutil.copy(os.path.join(HERE, f"{name}.cp"), dst)
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
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
