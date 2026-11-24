#!/usr/bin/env python3
"""Judge for YAC7 P5: answers "? e f" (1 if egg e shatters at floor f), within 2e5 queries,
then checks "! p". The moves (from floor 1, 20 more per reversal) only score, so they go to stderr."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

QUERIES = 200000


def drive(case_text, io):
    tokens = case_text.split()
    n = int(tokens[0])
    p = [int(x) for x in tokens[1:1 + n]]
    io.send(str(n))
    asked, moves, at, heading = 0, 0, 1, 0
    while True:
        parts = io.recv_line().split()
        if parts and parts[0] == "?" and len(parts) == 3:
            asked += 1
            e, f = int(parts[1]), int(parts[2])
            if asked > QUERIES or not (1 <= e <= n and 1 <= f <= n):
                io.send("-1")
                raise RuntimeError(f"bad or extra query {parts}")
            moves += abs(at - f) + 1
            if f != at:
                way = 1 if f > at else -1
                if heading and way != heading:
                    moves += 20
                heading, at = way, f
            io.send("1" if f >= p[e - 1] else "0")
        elif parts and parts[0] == "!":
            if [int(x) for x in parts[1:]] != p:
                raise RuntimeError("wrong permutation")
            sys.stderr.write(f"{moves} moves\n")
            return
        else:
            io.send("-1")
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
