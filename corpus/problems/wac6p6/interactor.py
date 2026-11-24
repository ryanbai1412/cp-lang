#!/usr/bin/env python3
"""Judge for WAC6 P6: per factory sends "A B C", then each cane's colour, taking EMPTY k any
number of times and then PLACE k or DISCARD; a bowl holds one colour. Full score needs at least
43 canes kept in the bowls of every factory."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

KEEP = 43


def drive(case_text, io):
    lines = case_text.split("\n")
    t = int(lines[0])
    io.send(str(t))
    for f in range(t):
        counts, canes = lines[1 + 2 * f], lines[2 + 2 * f].strip()
        io.send(counts)
        colour, held = [None, None, None], [0, 0, 0]
        for cane in canes:
            io.send(cane)
            while True:
                parts = io.recv_line().split()
                if parts == ["DISCARD"]:
                    break
                if len(parts) != 2 or parts[0] not in ("EMPTY", "PLACE") or parts[1] not in ("1", "2"):
                    io.send("-1")
                    raise RuntimeError(f"factory {f + 1}: bad operation {parts}")
                k = int(parts[1])
                if parts[0] == "EMPTY":
                    colour[k], held[k] = None, 0
                    continue
                if colour[k] not in (None, cane):
                    io.send("-1")
                    raise RuntimeError(f"factory {f + 1}: {cane} into bowl {k} of {colour[k]}")
                colour[k], held[k] = cane, held[k] + 1
                break
        if held[1] + held[2] < KEEP:
            raise RuntimeError(f"factory {f + 1}: kept {held[1] + held[2]}, fewer than {KEEP}")


if __name__ == "__main__":
    run_interactor(drive)
