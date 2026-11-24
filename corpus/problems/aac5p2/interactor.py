#!/usr/bin/env python3
"""Judge for AAC5 P2: answers "? i j" with a_i * a_j, at most n - 1 times, then checks "! a"."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    tokens = case_text.split()
    n = int(tokens[0])
    a = [int(x) for x in tokens[1:1 + n]]
    io.send(str(n))
    asked = 0
    while True:
        parts = io.recv_line().split()
        if parts[0] == "?":
            asked += 1
            if asked > n - 1:
                raise RuntimeError(f"more than {n - 1} queries")
            i, j = int(parts[1]), int(parts[2])
            if not (1 <= i <= n and 1 <= j <= n and i != j):
                raise RuntimeError(f"bad query {parts}")
            io.send(str(a[i - 1] * a[j - 1]))
        elif parts[0] == "!":
            if [int(x) for x in parts[1:]] != a:
                raise RuntimeError("wrong permutation")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
