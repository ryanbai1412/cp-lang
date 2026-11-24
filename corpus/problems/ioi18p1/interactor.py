#!/usr/bin/env python3
# Judge for Combo: sends N; `press p` (|p| <= 4N) gets the length of the longest prefix of S
# that is a substring of p; `guess S` ends the run, at most N + 2 presses (full score).
# The case is S.
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def score(word, p):
    lo, hi = 0, min(len(word), len(p))
    while lo < hi:
        mid = (lo + hi + 1) // 2
        lo, hi = (mid, hi) if word[:mid] in p else (lo, mid - 1)
    return lo


def drive(case_text, io):
    word = case_text.split()[0]
    n = len(word)
    io.send(str(n))
    presses = 0
    while True:
        parts = io.recv_line().split()
        if parts[:1] == ["press"]:
            p = parts[1] if len(parts) > 1 else ""
            if len(p) > 4 * n or set(p) - set("ABXY"):
                raise RuntimeError(f"bad press {p!r}")
            presses += 1
            if presses > n + 2:
                raise RuntimeError(f"over {n + 2} presses")
            io.send(str(score(word, p)))
        elif parts[:1] == ["guess"] and len(parts) == 2:
            if parts[1] != word:
                raise RuntimeError(f"wrong guess {parts[1]!r}")
            sys.stderr.write(f"presses {presses}\n")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
