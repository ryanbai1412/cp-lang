#!/usr/bin/env python3
# Judge for IOI 2010 Hotter Colder, stdio-adapted: sends Q, then per game N; the program
# prints `guess G` (1 <= G <= N), answered 0 the first time and then 1 (hotter), -1 (colder)
# or 0 (same) against the previous guess, and ends the game with `answer X`. A game may use
# at most 16 guesses for N <= 500 (subtask 3) and fewer than 2W for larger N, W the largest
# integer with 2^W <= 3N (subtask 4's scoring bar). The case is Q, then Q lines `N secret`.
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    q, games = t[0], list(zip(t[1::2], t[2::2]))
    if len(games) != q or not all(1 <= s <= n <= 500_000_000 for n, s in games):
        raise RuntimeError("bad case")
    io.send(str(q))
    most = 0
    for k, (n, s) in enumerate(games):
        w = (3 * n).bit_length() - 1
        limit = 16 if n <= 500 else 2 * w - 1
        io.send(str(n))
        prev, used = None, 0
        while True:
            parts = io.recv_line().split()
            if len(parts) != 2 or parts[0] not in ("guess", "answer"):
                raise RuntimeError(f"game {k + 1}: bad line {' '.join(parts)!r}")
            x = int(parts[1])
            if parts[0] == "answer":
                if x != s:
                    raise RuntimeError(f"game {k + 1}: answered {x}, Jill had {s}")
                break
            used += 1
            if not 1 <= x <= n or used > limit:
                raise RuntimeError(f"game {k + 1}: guess {used} is {x} (N = {n}, limit {limit})")
            reply = 0 if prev is None else (abs(x - s) < abs(prev - s)) - (abs(x - s) > abs(prev - s))
            prev = x
            io.send(str(reply))
        most = max(most, used)
    sys.stderr.write(f"most guesses in a game: {most}\n")


run_interactor(drive)
