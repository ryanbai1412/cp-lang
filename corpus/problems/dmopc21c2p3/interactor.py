#!/usr/bin/env python3
"""Judge for DMOPC '21 C2 P3: sends n, answers each guess G for F_i with the tree distance
between G and F_i (parent of x is x >> 1); 0 moves on to F_(i+1). At most 32000 guesses."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def distance(a, b):
    d = 0
    while a != b:
        if a > b:
            a >>= 1
        else:
            b >>= 1
        d += 1
    return d


def drive(case_text, io):
    tokens = case_text.split()
    n = int(tokens[0])
    hidden = [int(x) for x in tokens[1:1 + n]]
    io.send(str(n))
    guesses = 0
    for f in hidden:
        while True:
            g = int(io.recv_line())
            guesses += 1
            if guesses > 32000:
                raise RuntimeError("more than 32000 guesses")
            if not 0 <= g < 1 << 60:
                raise RuntimeError(f"guess {g} out of range")
            d = distance(f, g)
            io.send(str(d))
            if d == 0:
                break


if __name__ == "__main__":
    run_interactor(drive)
