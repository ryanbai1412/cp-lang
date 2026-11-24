#!/usr/bin/env python3
# Judge for BOI '18 Worm Worries: the program reads "N M K Q", asks "? X Y Z" (the humidity of
# that cell) at most Q times inside the box, then prints "! X Y Z" naming a cell no lower than
# any of its up to six neighbours. The case is "N M K Q" and the humidities (gen.py).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    nums = list(map(int, case_text.split()))
    n, m, k, q = nums[:4]
    h = nums[4:]

    def at(x, y, z):
        if not (1 <= x <= n and 1 <= y <= m and 1 <= z <= k):
            return 0
        return h[((x - 1) * m + (y - 1)) * k + (z - 1)]

    io.send(f"{n} {m} {k} {q}")
    asks = 0
    while True:
        parts = io.recv_line().split()
        if len(parts) != 4 or parts[0] not in ("?", "!"):
            raise RuntimeError(f"unexpected {parts}")
        x, y, z = map(int, parts[1:])
        if not (1 <= x <= n and 1 <= y <= m and 1 <= z <= k):
            raise RuntimeError(f"{parts[0]} {x} {y} {z} is outside the box")
        if parts[0] == "?":
            asks += 1
            if asks > q:
                raise RuntimeError(f"more than {q} questions")
            io.send(str(at(x, y, z)))
            continue
        here = at(x, y, z)
        for dx, dy, dz in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)):
            if at(x + dx, y + dy, z + dz) > here:
                raise RuntimeError(f"! {x} {y} {z} has a higher neighbour")
        sys.stderr.write(f"questions {asks}\n")
        return


if __name__ == "__main__":
    run_interactor(drive)
