#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    toks = list(map(int, case_text.split()))
    it = iter(toks)
    tc = next(it)
    io.send(str(tc))
    for _ in range(tc):
        n = next(it)
        p = [next(it) for _ in range(n)]
        pos = [0] * (n + 1)
        for idx, val in enumerate(p, 1):
            pos[val] = idx
        io.send(str(n))
        total_q = 0
        big_q = 0
        while True:
            parts = io.recv_line().split()
            if parts[0] == "!":
                ans = [int(x) for x in parts[1:]]
                if ans != p:
                    raise RuntimeError(f"wrong permutation: got {ans[:10]} != {p[:10]}")
                break
            qt = int(parts[0])
            if qt not in (1, 2, 3, 4):
                raise RuntimeError(f"invalid query type {qt}")
            total_q += 1
            if qt in (3, 4):
                big_q += 1
            if total_q > 30:
                raise RuntimeError(f"too many queries: {total_q} > 30")
            if big_q > 1:
                raise RuntimeError(f"too many big queries: {big_q} > 1")
            m = int(parts[1])
            arr = [int(x) for x in parts[2:]]
            if len(arr) != m or m < 1 or m > n:
                raise RuntimeError(f"bad query len {len(arr)} vs {m} (n={n})")
            if len(set(arr)) != m or any(x < 1 or x > n for x in arr):
                raise RuntimeError("duplicate or out-of-range query elements")
            k = min(300 if qt in (3, 4) else 60, m)
            if qt in (1, 3):
                vals = sorted(p[x - 1] for x in arr)
            else:
                vals = sorted(pos[x] for x in arr)
            io.send(" ".join(map(str, vals[-k:])))


if __name__ == "__main__":
    run_interactor(drive)
