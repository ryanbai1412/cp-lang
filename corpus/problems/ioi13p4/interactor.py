#!/usr/bin/env python3
# Judge for Cave: sends N; `try s_0 .. s_{N-1}` (each 0 or 1) gets the first closed door, or -1
# when all are open, at most 70 000 tries; `answer S_0 .. S_{N-1} D_0 .. D_{N-1}` ends the run.
# The case is N, each switch's open position, each switch's door.
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 70_000


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    n = t[0]
    open_, door = t[1 : 1 + n], t[1 + n : 1 + 2 * n]
    io.send(str(n))
    tries = 0
    while True:
        parts = io.recv_line().split()
        if parts[:1] == ["try"] and len(parts) == n + 1:
            s = list(map(int, parts[1:]))
            tries += 1
            if tries > LIMIT:
                raise RuntimeError(f"over {LIMIT} tries")
            io.send(str(min((d for d, x, o in zip(door, s, open_) if x != o), default=-1)))
        elif parts[:1] == ["answer"] and len(parts) == 2 * n + 1:
            got = list(map(int, parts[1:]))
            if got != open_ + door:
                raise RuntimeError("wrong answer")
            sys.stderr.write(f"tries {tries}\n")
            return
        else:
            raise RuntimeError(f"unexpected {parts[:3]}")


if __name__ == "__main__":
    run_interactor(drive)
