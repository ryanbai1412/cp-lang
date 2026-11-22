#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    toks = iter(case_text.split())
    t = int(next(toks))
    io.send(str(t))
    for _ in range(t):
        n = int(next(toks))
        m = n * n + 1
        p = [int(next(toks)) for _ in range(m)]
        io.send(str(n))
        queries = 0
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                k = int(parts[1])
                idxs = [int(x) for x in parts[2:]]
                if len(idxs) != k or idxs != sorted(set(idxs)):
                    raise RuntimeError(f"bad query {parts}")
                queries += 1
                if queries > n:
                    raise RuntimeError("too many queries")
                mx = -1
                res = []
                for i in idxs:
                    if p[i - 1] > mx:
                        mx = p[i - 1]
                        res.append(i)
                io.send(f"{len(res)} " + " ".join(map(str, res)))
            elif parts[0] == "!":
                got = [int(x) for x in parts[1:]]
                if len(got) != n + 1 or got != sorted(set(got)):
                    raise RuntimeError(f"bad answer indices {got}")
                seq = [p[i - 1] for i in got]
                inc = all(seq[i] < seq[i + 1] for i in range(n))
                dec = all(seq[i] > seq[i + 1] for i in range(n))
                if not (inc or dec):
                    raise RuntimeError(f"not monotone: {seq} from {got}")
                break
            else:
                raise RuntimeError(f"bad command {parts}")


run_interactor(drive)
