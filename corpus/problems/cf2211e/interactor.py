#!/usr/bin/env python3
"""Online judge for CF 2211 E: `interactor.py CASE [-- SOLUTION...]`.

CASE is the full input. Node lines are sent one at a time: the next only after
the solution has printed its answer for the previous node, so a solution that
reads ahead blocks. If CASE ends in `.in` and the matching `.ans` or `.out`
exists, every answer must equal it."""
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor

CASE = sys.argv[1] if len(sys.argv) > 1 else ""


def expected_answers():
    for ext in (".ans", ".out"):
        if CASE.endswith(".in") and os.path.exists(CASE[:-3] + ext):
            return open(CASE[:-3] + ext, encoding="utf-8").read().split()
    return None


def drive(text, io):
    lines = [l.strip() for l in text.splitlines() if l.strip()]
    expected = expected_answers()
    t, pos, got = int(lines[0]), 1, []
    io.send(lines[0])
    for c in range(t):
        n = int(lines[pos])
        io.send(lines[pos])
        for i in range(n):
            io.send(lines[pos + 1 + i])
            ans = io.recv_line().split()
            if len(ans) != 1:
                raise RuntimeError(f"case {c + 1}, line {i + 1}: want one answer, got {' '.join(ans)[:40]!r}")
            if expected is not None:
                want = expected[len(got)] if len(got) < len(expected) else None
                if ans[0] != want:
                    raise RuntimeError(f"case {c + 1}, line {i + 1}: answer {ans[0]}, want {want}")
            got.append(ans[0])
        pos += 1 + n
    if expected is not None and len(got) != len(expected):
        raise RuntimeError(f"{len(got)} answers, want {len(expected)}")


if __name__ == "__main__":
    run_interactor(drive)
