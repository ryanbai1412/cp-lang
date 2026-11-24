#!/usr/bin/env python3
# Judge for IOI 2010 Memory, stdio-adapted: the program prints a card C (1..50) for each
# faceup(C) and reads its letter; after every second card both turn face down again. It
# prints 0 when play returns, which must be after Jack has all 25 candies and within 100
# faceup calls (subtask 2). The case is the 50 letters of cards 1..50 (gen.py).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 100


def drive(case_text, io):
    cards = case_text.strip()
    if len(cards) != 50 or sorted(cards) != sorted("ABCDEFGHIJKLMNOPQRSTUVWXY" * 2):
        raise RuntimeError("bad case: each of A..Y twice")
    up, candies, calls = [], set(), 0
    while True:
        c = int(io.recv_line())
        if c == 0:
            break
        calls += 1
        if calls > LIMIT:
            raise RuntimeError(f"more than {LIMIT} faceup calls")
        if not 1 <= c <= 50 or c in up:
            raise RuntimeError(f"call {calls}: card {c} is not face down")
        up.append(c)
        io.send(cards[c - 1])
        if len(up) == 2:
            if cards[up[0] - 1] == cards[up[1] - 1]:
                candies.add(cards[c - 1])
            up = []
    if len(candies) != 25:
        raise RuntimeError(f"play returned with {len(candies)} candies")


if __name__ == "__main__":
    run_interactor(drive)
