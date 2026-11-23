#!/usr/bin/env python3
"""Online judge for CF 2211 E: `interactor.py CASE -- SOLUTION...`.

CASE is the full input. Node lines are sent one at a time: the next only after
the solution has printed its answer for the previous node, so a solution that
reads ahead blocks. If CASE ends in `.in` and the matching `.out` exists, every
answer must equal it."""
import os
import subprocess
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        sys.stderr.write("usage: interactor.py CASE -- SOLUTION...\n")
        sys.exit(2)
    case = sys.argv[1]
    lines = [l.strip() for l in open(case, encoding="utf-8") if l.strip()]
    expected = None
    if case.endswith(".in") and os.path.exists(case[:-3] + ".out"):
        expected = open(case[:-3] + ".out", encoding="utf-8").read().split()
    proc = subprocess.Popen(sys.argv[3:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def send(line):
        proc.stdin.write(line + "\n")
        proc.stdin.flush()

    t, pos, got = int(lines[0]), 1, []
    send(lines[0])
    for c in range(t):
        n = int(lines[pos])
        send(lines[pos])
        for i in range(n):
            send(lines[pos + 1 + i])
            ans = proc.stdout.readline().split()
            if len(ans) != 1:
                fail(f"case {c + 1}, line {i + 1}: want one answer, got {' '.join(ans)[:40]!r}")
            if expected is not None:
                want = expected[len(got)] if len(got) < len(expected) else None
                if ans[0] != want:
                    fail(f"case {c + 1}, line {i + 1}: answer {ans[0]}, want {want}")
            got.append(ans[0])
        pos += 1 + n
    proc.stdin.close()
    if proc.stdout.read().split():
        fail("solution printed extra output")
    if proc.wait(timeout=20) != 0:
        fail(f"solution exited {proc.returncode}")
    if expected is not None and len(got) != len(expected):
        fail(f"{len(got)} answers, want {len(expected)}")


if __name__ == "__main__":
    main()
