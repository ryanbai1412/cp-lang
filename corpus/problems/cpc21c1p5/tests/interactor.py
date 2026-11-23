#!/usr/bin/env python3
# usage: interactor.py <case> -- <solution...>; a case is n, then n lines "x1 x2 y1 y2"
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT, QUERIES = 10**9, 61000


def drive(case_text, io):
    toks = list(map(int, case_text.split()))
    n = toks[0]
    rects = [tuple(toks[1 + 4 * i:5 + 4 * i]) for i in range(n)]
    io.send(str(n))
    asked = 0
    while True:
        line = io.recv_line().split()
        if line == ["!"]:
            break
        if len(line) != 5 or line[0] != "?":
            raise RuntimeError(f"bad line {line}")
        a, b, c, d = map(int, line[1:])
        if not (1 <= a <= b <= LIMIT and 1 <= c <= d <= LIMIT):
            raise RuntimeError(f"bad query {line}")
        asked += 1
        if asked > QUERIES:
            raise RuntimeError("more than 61000 queries")
        io.send(str(sum(a <= x1 and x2 <= b and c <= y1 and y2 <= d for x1, x2, y1, y2 in rects)))
    got = [tuple(map(int, io.recv_line().split())) for _ in range(n)]
    if sorted(got) != sorted(rects):
        raise RuntimeError("wrong rectangles")
    sys.stderr.write(f"ok {asked} queries\n")


run_interactor(drive)
