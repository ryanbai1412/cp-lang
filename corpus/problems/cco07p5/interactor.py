#!/usr/bin/env python3
# Judge for the DMOJ protocol: GetSize -> N; Look a b -> True/False, then the particle
# moves v; Guess v p ends the run. The case is "N v p" (initial position p).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 100_000


def drive(case_text, io):
    n, v, p = map(int, case_text.split())
    looks = 0
    while True:
        parts = io.recv_line().split()
        if parts == ["GetSize"]:
            io.send(str(n))
        elif parts[0] == "Look":
            a, b = int(parts[1]), int(parts[2])
            if not 0 <= a <= b <= n - 1:
                raise RuntimeError(f"bad look {a} {b}")
            looks += 1
            if looks > LIMIT:
                raise RuntimeError("too many looks")
            io.send("True" if a <= p <= b else "False")
            p = (p + v) % n
        elif parts[0] == "Guess":
            if (int(parts[1]), int(parts[2])) != (v, p):
                raise RuntimeError(f"wrong guess {parts[1:]} != {v} {p}")
            sys.stderr.write(f"looks {looks}\n")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
