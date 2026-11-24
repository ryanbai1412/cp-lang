#!/usr/bin/env python3
# Judge for IOI '07 Aliens: the program reads "N X0 Y0", asks "examine X Y" (true when
# flattened) at most 300 times inside the meadow, then prints "solution XC YC". The case
# is "N M BX BY X0 Y0" (gen.py).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 300


def drive(case_text, io):
    n, m, bx, by, x0, y0 = map(int, case_text.split())

    def flat(x, y):
        i, j = (x - bx) // m, (y - by) // m
        return 0 <= i < 5 and 0 <= j < 5 and (i + j) % 2 == 0

    if not flat(x0, y0):
        raise RuntimeError(f"bad case: ({x0}, {y0}) is not flattened")
    io.send(f"{n} {x0} {y0}")
    asks = 0
    while True:
        parts = io.recv_line().split()
        if len(parts) == 3 and parts[0] == "examine":
            x, y = int(parts[1]), int(parts[2])
            asks += 1
            if not (1 <= x <= n and 1 <= y <= n):
                raise RuntimeError(f"examine {x} {y} is outside the meadow")
            if asks > LIMIT:
                raise RuntimeError("more than 300 examines")
            io.send("true" if flat(x, y) else "false")
        elif len(parts) == 3 and parts[0] == "solution":
            want = (bx + 2 * m + m // 2, by + 2 * m + m // 2)
            if (int(parts[1]), int(parts[2])) != want:
                raise RuntimeError(f"solution {parts[1]} {parts[2]}, want {want[0]} {want[1]}")
            sys.stderr.write(f"examines {asks}\n")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
