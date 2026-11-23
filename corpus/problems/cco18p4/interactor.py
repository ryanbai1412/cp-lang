#!/usr/bin/env python3
"""Judge for CCO 2018 P4: `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is `R C K`, then `N`, then N lines `X Y`. Answers `? p q` with the cell's score and checks
the final `! Z` against the true minimum, within K questions."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    r, c, k, n = t[:4]
    xs, ys = t[4:4 + 2 * n:2], t[5:5 + 2 * n:2]
    score = lambda p, q: sum(abs(p - x) for x in xs) + sum(abs(q - y) for y in ys)
    sx, sy = sorted(xs), sorted(ys)
    best = score(sx[(n - 1) // 2], sy[(n - 1) // 2])
    io.send(f"{r} {c} {k}")
    asked = 0
    while True:
        w = io.recv_line().split()
        if len(w) == 2 and w[0] == "!":
            if int(w[1]) != best:
                raise RuntimeError(f"answer {w[1]}, want {best}")
            break
        if len(w) != 3 or w[0] != "?":
            raise RuntimeError(f"bad line {' '.join(w)[:40]!r}")
        p, q = int(w[1]), int(w[2])
        if not (1 <= p <= r and 1 <= q <= c):
            raise RuntimeError(f"cell ({p}, {q}) outside {r} x {c}")
        asked += 1
        if asked > k:
            raise RuntimeError(f"more than {k} questions")
        io.send(str(score(p, q)))
    sys.stderr.write(f"ok: {asked} questions\n")


run_interactor(drive)
