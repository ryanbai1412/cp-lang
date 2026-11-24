#!/usr/bin/env python3
"""Judge for EGOI 2023 "Find the Box": `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is `H W R C`: the grid and the box. Sends `H W`. `? s` (at most 20000 of <>^v) moves the
robot from (0, 0), stopping at walls and the box, and answers where it ends. `! r c` must name
the box, after at most 2 queries (full score)."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor

STEP = {"<": (0, -1), ">": (0, 1), "^": (-1, 0), "v": (1, 0)}


def drive(case_text, io):
    h, w, br, bc = map(int, case_text.split())
    io.send(f"{h} {w}")
    asked = 0
    while True:
        words = io.recv_line().split()
        if words[:1] == ["?"] and len(words) <= 2:
            asked += 1
            if asked > 2:
                raise RuntimeError("more than 2 queries")
            moves = words[1] if len(words) == 2 else ""
            if len(moves) > 20000 or set(moves) - set(STEP):
                raise RuntimeError(f"bad moves {moves[:40]!r}")
            r = c = 0
            for m in moves:
                nr, nc = r + STEP[m][0], c + STEP[m][1]
                if 0 <= nr < h and 0 <= nc < w and (nr, nc) != (br, bc):
                    r, c = nr, nc
            io.send(f"{r} {c}")
        elif words[:1] == ["!"] and len(words) == 3:
            if (int(words[1]), int(words[2])) != (br, bc):
                raise RuntimeError(f"answered {words[1]} {words[2]}, the box is at {br} {bc}")
            break
        else:
            raise RuntimeError(f"bad line {' '.join(words)[:40]!r}")
    sys.stderr.write(f"ok: {asked} queries\n")


run_interactor(drive)
