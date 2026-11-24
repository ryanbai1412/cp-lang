#!/usr/bin/env python3
# Judge for IOI 2011 Parrots, stdio-adapted: sends N and the message M; the program prints L
# then the L numbers its encode sends; they are shuffled and sent back as L and the numbers,
# and the program prints the N decoded integers. Every number is in [0, 255] and L is at most
# 10 N for N <= 32 (subtasks 1-4) or 15 N above (subtask 5). The case is N then M (gen.py);
# the shuffle is seeded by the case, so a run is deterministic.
import os, random, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def tokens(io, k):
    out = []
    while len(out) < k:
        out += io.recv_line().split()
    if len(out) != k:
        raise RuntimeError(f"expected {k} numbers, got {len(out)}")
    return [int(x) for x in out]


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    n, msg = t[0], t[1:]
    if not 1 <= n <= 64 or len(msg) != n or not all(0 <= x <= 255 for x in msg):
        raise RuntimeError("bad case")
    io.send(f"{n}\n{' '.join(map(str, msg))}")
    (l,) = tokens(io, 1)
    if not 0 <= l <= (10 if n <= 32 else 15) * n:
        raise RuntimeError(f"{l} parrots for N = {n}")
    birds = tokens(io, l) if l else []
    if not all(0 <= x <= 255 for x in birds):
        raise RuntimeError("a parrot carries a number outside [0, 255]")
    random.Random(case_text).shuffle(birds)
    io.send(f"{l}\n{' '.join(map(str, birds))}")
    if tokens(io, n) != msg:
        raise RuntimeError("decoded message differs")
    sys.stderr.write(f"parrots {l} for N = {n}\n")


run_interactor(drive)
