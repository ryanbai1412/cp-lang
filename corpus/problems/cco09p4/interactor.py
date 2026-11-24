#!/usr/bin/env python3
"""Judge for CCO 2009 P4: `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is n, the n cap sizes, then the n bottle sizes. Sends n, answers `0 c b` with 1, 0 or -1
(cap too large, fits, too small) and checks that the n `1 c b` reports are the true matches,
within 5e5 queries."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    n = t[0]
    cap, bottle = t[1:1 + n], t[1 + n:1 + 2 * n]
    io.send(str(n))
    asked, matched = 0, set()
    while len(matched) < n:
        w = io.recv_line().split()
        if len(w) != 3 or w[0] not in ("0", "1"):
            raise RuntimeError(f"bad line {' '.join(w)[:40]!r}")
        c, b = int(w[1]), int(w[2])
        if not (1 <= c <= n and 1 <= b <= n):
            raise RuntimeError(f"cap {c} or bottle {b} outside 1..={n}")
        x, y = cap[c - 1], bottle[b - 1]
        if w[0] == "0":
            asked += 1
            if asked > 500_000:
                raise RuntimeError("more than 500000 queries")
            io.send("1" if x > y else "0" if x == y else "-1")
        else:
            if x != y or c in matched:
                raise RuntimeError(f"wrong or repeated match {c} {b}")
            matched.add(c)
    sys.stderr.write(f"ok: {asked} queries\n")


run_interactor(drive)
