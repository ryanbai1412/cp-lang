#!/usr/bin/env python3
"""Judge for NOI 2002 P2: answers GetNM and "Ask a b T" as child a would (Star truthful, Moon
lying, the referee truthful on his odd-numbered answers), each asker asking about a child once,
then checks the N + M + 1 "Answer x" lines and the full-marks rule of under three questions each."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    tokens = [int(x) for x in case_text.split()]
    stars, moons = tokens[0], tokens[1]
    n = stars + moons + 1
    role = tokens[2:2 + n]  # 0 Star, 1 Moon, 2 referee
    if io.recv_line() != "GetNM":
        raise RuntimeError("GetNM must come first")
    io.send(f"{stars} {moons}")
    asked, pairs = [0] * n, set()
    while True:
        parts = io.recv_line().split()
        if parts and parts[0] == "Ask" and len(parts) == 4:
            a, b, t = int(parts[1]) - 1, int(parts[2]) - 1, int(parts[3])
            if not (0 <= a < n and 0 <= b < n and a != b and t in (0, 1)) or (a, b) in pairs:
                raise RuntimeError(f"bad query {parts}")
            pairs.add((a, b))
            asked[a] += 1
            truth = role[b] == t
            honest = role[a] == 0 or (role[a] == 2 and asked[a] % 2 == 1)
            io.send("1" if truth == honest else "0")
        elif parts and parts[0] == "Answer":
            got = [int(parts[1])] + [int(io.recv_line().split()[1]) for _ in range(n - 1)]
            if got != role:
                raise RuntimeError("wrong roles")
            if max(asked) >= 3:
                raise RuntimeError(f"a child was asked {max(asked)} questions")
            return
        else:
            raise RuntimeError(f"unexpected {parts}")


if __name__ == "__main__":
    run_interactor(drive)
