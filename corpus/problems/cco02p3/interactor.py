#!/usr/bin/env python3
"""Judge for CCO 2002 P3: `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is gen.py's board. Sends `n m` and `r c f`, answers `F` with 1 (moved) or 0 (wall or
obstacle), applies `L` and `R`, then after `A` checks the n map rows (spaces ignored)."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor

STEP = [(-1, 0), (0, 1), (1, 0), (0, -1)]  # N E S W


def drive(case_text, io):
    lines = case_text.split("\n")
    n, m = map(int, lines[0].split())
    r, c, f = lines[1].split()
    r, c, d = int(r) - 1, int(c) - 1, "NESW".index(f)
    board = lines[2:2 + n]
    io.send(f"{n} {m}")
    io.send(f"{r + 1} {c + 1} {f}")
    moves = 0
    while True:
        w = io.recv_line()
        moves += 1
        if moves > 1_000_000:
            raise RuntimeError("more than 1e6 moves")
        if w == "L":
            d = (d + 3) % 4
        elif w == "R":
            d = (d + 1) % 4
        elif w == "F":
            a, b = r + STEP[d][0], c + STEP[d][1]
            ok = 0 <= a < n and 0 <= b < m and board[a][b] == "."
            if ok:
                r, c = a, b
            io.send("1" if ok else "0")
        elif w == "A":
            break
        else:
            raise RuntimeError(f"bad line {w[:40]!r}")
    for x in range(n):
        got = io.recv_line().replace(" ", "")
        if got != board[x]:
            raise RuntimeError(f"row {x + 1}: got {got[:80]!r}, want {board[x]!r}")
    sys.stderr.write(f"ok: {moves} moves\n")


run_interactor(drive)
