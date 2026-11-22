#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def sim(a, t, seq):
    items = list(zip(a, t))
    for ch in seq:
        if ch == "R":
            nxt = []
            for i in range(len(items) - 1, -1, -1):
                p, spd = items[i]
                np = p + spd
                if nxt and np >= nxt[-1][0]:
                    continue
                nxt.append((np, spd))
            items = nxt[::-1]
        elif ch == "L":
            nxt = []
            for p, spd in items:
                np = p - spd
                if nxt and np <= nxt[-1][0]:
                    continue
                nxt.append((np, spd))
            items = nxt
        else:
            raise RuntimeError(f"bad move {ch}")
    return [p for p, _ in items]


def is_ambiguous(a, t):
    n = len(a)
    for i in range(1, n - 1):
        if t[i] in (1, 2) and t[i - 1] == 0 and t[i + 1] == 0 and a[i - 1] == a[i] - 1 and a[i + 1] == a[i] + 1:
            return True
    return False


def drive(case_text, io):
    toks = list(map(int, case_text.split()))
    it = iter(toks)
    tc = next(it)
    io.send(str(tc))
    for _ in range(tc):
        n = next(it)
        a = [next(it) for _ in range(n)]
        t = [next(it) for _ in range(n)]
        io.send(str(n))
        io.send(" ".join(map(str, a)))
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                res = sim(a, t, parts[1])
                io.send(str(len(res)))
                if res:
                    io.send(" ".join(map(str, res)))
            elif parts[0] == "!":
                vals = list(map(int, parts[1:]))
                if is_ambiguous(a, t):
                    if vals != [-1]:
                        raise RuntimeError(f"expected -1 for ambiguous case, got {vals}")
                else:
                    if vals != t:
                        raise RuntimeError(f"wrong answer {vals} != {t} for a={a}")
                break
            else:
                raise RuntimeError(f"unexpected cmd {parts}")


if __name__ == "__main__":
    run_interactor(drive)
