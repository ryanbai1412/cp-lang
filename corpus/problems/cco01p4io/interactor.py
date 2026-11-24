#!/usr/bin/env python3
"""Judge for CCO 2001 P4: `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is `n d h`. Answers `GetN`, `GetD`, and `Drop t` (1 if t >= h, so the sample breaks);
two samples, at most d drops; checks the final `Decide x` against h."""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    n, d, h = map(int, case_text.split())
    drops = broken = 0
    while True:
        w = io.recv_line().split()
        if w == ["GetN"]:
            io.send(str(n))
        elif w == ["GetD"]:
            io.send(str(d))
        elif len(w) == 2 and w[0] == "Drop":
            t = int(w[1])
            drops += 1
            if not 1 <= t <= n or drops > d or broken == 2:
                raise RuntimeError(f"drop {drops} at {t}: outside 1..={n}, over {d}, or no sample left")
            broken += t >= h
            io.send("1" if t >= h else "0")
        elif len(w) == 2 and w[0] == "Decide":
            if int(w[1]) != h:
                raise RuntimeError(f"decided {w[1]}, want {h}")
            break
        else:
            raise RuntimeError(f"bad line {' '.join(w)[:40]!r}")
    sys.stderr.write(f"ok: {drops} drops\n")


run_interactor(drive)
