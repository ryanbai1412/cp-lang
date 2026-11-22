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
        w = next(it)
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                k = int(parts[1])
                arr = [int(x) for x in parts[2:]]
                if len(arr) != k:
                    raise RuntimeError(f"bad len {len(arr)} != {k}")
                if max(arr) > w:
                    io.send("0")
                else:
                    lines = 1
                    cur = 0
                    for x in arr:
                        if cur + x <= w:
                            cur += x
                        else:
                            lines += 1
                            cur = x
                    io.send(str(lines))
            elif parts[0] == "!":
                ans = int(parts[1])
                if ans != w:
                    raise RuntimeError(f"wrong {ans} != {w}")
                break
            else:
                raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
