#!/usr/bin/env python3
import os, sys
from collections import Counter
sys.path.insert(0, os.path.dirname(__file__))
from _lib import run_interactor


def drive(case_text, io):
    toks = iter(case_text.split())
    t = int(next(toks))
    io.send(str(t))
    for _ in range(t):
        n = int(next(toks))
        a = [int(next(toks)) for _ in range(2 * n)]
        io.send(str(n))
        queries = 0
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                k = int(parts[1])
                idxs = [int(x) for x in parts[2:]]
                if len(idxs) != k or len(set(idxs)) != k:
                    raise RuntimeError(f"bad query {parts}")
                queries += 1
                if queries > 3 * n:
                    raise RuntimeError("too many queries")
                cnt = Counter(a[i - 1] for i in idxs)
                mad = max((v for v, c in cnt.items() if c >= 2), default=0)
                io.send(str(mad))
            elif parts[0] == "!":
                got = [int(x) for x in parts[1:]]
                if got != a:
                    raise RuntimeError(f"wrong answer {got}, expected {a}")
                break
            else:
                raise RuntimeError(f"bad command {parts}")


run_interactor(drive)
