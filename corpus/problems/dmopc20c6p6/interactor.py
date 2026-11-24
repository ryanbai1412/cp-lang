#!/usr/bin/env python3
"""Judge for DMOPC '20 C6 P6: sends "n m"; "E i" empties, "F i" fills, "D i j" dumps i
into j and answers how much j rose; at most 1e5 operations, then "A c_1 ... c_n"."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    tokens = [int(x) for x in case_text.split()]
    n, m = tokens[0], tokens[1]
    cap = tokens[2:2 + n]
    water = cap[:]
    io.send(f"{n} {m}")
    ops = 0
    while True:
        parts = io.recv_line().split()
        if parts[0] == "A":
            if [int(x) for x in parts[1:]] != cap:
                raise RuntimeError("wrong capacities")
            return
        ops += 1
        if ops > 100000:
            raise RuntimeError("more than 1e5 operations")
        idx = [int(x) - 1 for x in parts[1:]]
        if any(not 0 <= i < n for i in idx):
            raise RuntimeError(f"bad operation {parts}")
        if parts[0] == "E" and len(idx) == 1:
            water[idx[0]] = 0
        elif parts[0] == "F" and len(idx) == 1:
            water[idx[0]] = cap[idx[0]]
        elif parts[0] == "D" and len(idx) == 2:
            x, y = idx
            moved = min(cap[y] - water[y], water[x])
            water[x] = 0
            water[y] += moved
            io.send(str(moved))
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
