#!/usr/bin/env python3
# Judge for SAC '22 CC4 P5: sends "N Q1 Q2 T", answers "c i j" with the taller index and
# "h k" with H_k, at most Q1 and Q2 of each, and accepts "! H_1 .. H_N" when it matches.
# A malformed or over-budget query gets -1 and fails. The case is "N Q1 Q2 T" then H (gen.py).
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    head, perm = case_text.strip().split("\n")
    n, q1, q2, t = map(int, head.split())
    h = [0] + list(map(int, perm.split()))
    if sorted(h[1:]) != list(range(1, n + 1)):
        raise RuntimeError("bad case: H is not a permutation")
    io.send(head)
    used = {"c": 0, "h": 0}
    while True:
        parts = io.recv_line().split()
        if parts and parts[0] == "!":
            if list(map(int, parts[1:])) != h[1:]:
                raise RuntimeError("wrong heights")
            return
        ok = parts and parts[0] in used and all(p.lstrip("-").isdigit() for p in parts[1:])
        if ok:
            idx = list(map(int, parts[1:]))
            if parts[0] == "c":
                ok = len(idx) == 2 and idx[0] != idx[1] and all(1 <= i <= n for i in idx) and used["c"] < q1
            else:
                ok = len(idx) == 1 and 1 <= idx[0] <= n and used["h"] < q2
        if not ok:
            io.send("-1")
            raise RuntimeError(f"bad or over-budget query {' '.join(parts)!r}")
        used[parts[0]] += 1
        if parts[0] == "c":
            i, j = idx
            io.send(str(i if h[i] > h[j] else j))
        else:
            io.send(str(h[idx[0]]))


if __name__ == "__main__":
    run_interactor(drive)
