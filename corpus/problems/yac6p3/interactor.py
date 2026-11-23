#!/usr/bin/env python3
"""Judge for YAC 6 P3: `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is `N M`, then `A B C D`, the two distinct hidden cells. Answers `? X Y` with the Manhattan
distance to the nearer cell (-1 and a failure on a cell outside the grid or a sixth query) and
checks the final `! A B C D` against the two cells in either order. Not adaptive."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor

LIMIT = 5


def drive(case_text, io):
    n, m, a, b, c, d = map(int, case_text.split())
    assert 1 <= n <= 10**9 and 1 <= m <= 10**9
    assert 1 <= a <= n and 1 <= c <= n and 1 <= b <= m and 1 <= d <= m and (a, b) != (c, d)
    io.send(f"{n} {m}")
    asked = 0
    while True:
        w = io.recv_line().split()
        if len(w) == 5 and w[0] == "!":
            got = sorted([(int(w[1]), int(w[2])), (int(w[3]), int(w[4]))])
            if got != sorted([(a, b), (c, d)]):
                raise RuntimeError(f"answer {' '.join(w[1:])}, want {a} {b} {c} {d}")
            break
        if len(w) != 3 or w[0] != "?":
            raise RuntimeError(f"bad line {' '.join(w)[:40]!r}")
        x, y = int(w[1]), int(w[2])
        asked += 1
        if not (1 <= x <= n and 1 <= y <= m) or asked > LIMIT:
            io.send("-1")
            raise RuntimeError(f"query {asked} at ({x}, {y}) is invalid or over {LIMIT}")
        io.send(str(min(abs(x - a) + abs(y - b), abs(x - c) + abs(y - d))))
    sys.stderr.write(f"ok: {asked} questions\n")


run_interactor(drive)
