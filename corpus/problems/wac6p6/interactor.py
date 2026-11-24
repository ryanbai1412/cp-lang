#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    lines = case_text.split()
    t = int(lines[0])
    io.send(str(t))
    for f in range(t):
        counts, canes = lines[1 + 4 * f: 4 + 4 * f], lines[4 + 4 * f]
        io.send(" ".join(counts))
        bowls, kept = {1: None, 2: None}, 0
        for cane in canes:
            io.send(cane)
            move = io.recv_line().split()
            if move[0] == "EMPTY":
                bowls[int(move[1])] = None
                move = io.recv_line().split()
            if move[0] == "PLACE":
                b = int(move[1])
                if bowls.get(b, cane) not in (None, cane):
                    raise RuntimeError(f"factory {f + 1}: {cane} into bowl {b} of {bowls[b]}")
                bowls[b], kept = cane, kept + 1
            elif move[0] != "DISCARD":
                raise RuntimeError(f"factory {f + 1}: unexpected {move}")
        if kept < 43:
            raise RuntimeError(f"factory {f + 1}: kept {kept} < 43")


if __name__ == "__main__":
    run_interactor(drive)
