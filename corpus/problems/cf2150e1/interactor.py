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
        arr = [next(it) for _ in range(2 * n - 1)]
        cnt = {}
        for x in arr:
            cnt[x] = cnt.get(x, 0) + 1
        single = [x for x, c in cnt.items() if c == 1][0]
        io.send(str(n))
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                x = int(parts[1])
                k = int(parts[2])
                idxs = list(map(int, parts[3:]))
                if len(idxs) != k:
                    raise RuntimeError(f"bad query len {len(idxs)} != {k}")
                found = any(arr[idx - 1] == x for idx in idxs)
                io.send("1" if found else "0")
            elif parts[0] == "!":
                ans = int(parts[1])
                if ans != single:
                    raise RuntimeError(f"wrong answer {ans} != {single}")
                break
            else:
                raise RuntimeError(f"unexpected cmd {parts}")


if __name__ == "__main__":
    run_interactor(drive)
