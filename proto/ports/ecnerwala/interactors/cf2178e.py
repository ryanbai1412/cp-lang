#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _lib import run_interactor


def drive(case_text, io):
    toks = iter(case_text.split())
    t = int(next(toks))
    io.send(str(t))
    for _ in range(t):
        n = int(next(toks))
        a = [int(next(toks)) for _ in range(n)]
        pref = [0] * (n + 1)
        for i, v in enumerate(a):
            pref[i + 1] = pref[i] + v
        io.send(str(n))
        queries = 0
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                l, r = int(parts[1]), int(parts[2])
                if not (1 <= l <= r <= n):
                    raise RuntimeError(f"bad range {l} {r} for n={n}")
                queries += 1
                if queries > 300:
                    raise RuntimeError("too many queries")
                io.send(str(pref[r] - pref[l - 1]))
            elif parts[0] == "!":
                ans = int(parts[1])
                if ans != max(a):
                    raise RuntimeError(f"wrong answer {ans}, expected {max(a)} for {a}")
                break
            else:
                raise RuntimeError(f"bad command {parts}")


run_interactor(drive)
