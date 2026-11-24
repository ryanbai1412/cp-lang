#!/usr/bin/env python3
"""Checker for IOI 2014 Gondola: tasks 1 and 3 must equal the answer; a task 2 replacement
sequence must turn some rotation of 1..n into the final numbers, replacing gondola g at step
g - n, with the length the answer has."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    task, n, seq = inp[0], inp[1], inp[2:2 + inp[1]]
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if task != 2:
        if out != ref:
            fail(f"got {' '.join(out[:1])}, want {' '.join(ref[:1])}")
        return
    if not out or int(out[0]) != int(ref[0]) or len(out) != 1 + int(out[0]):
        fail(f"want {ref[0]} replacements")
    broken = list(map(int, out[1:]))
    # the slot holding each final number; replay backwards from the final state
    where = {g: i for i, g in enumerate(seq)}
    now = list(seq)
    for step in range(len(broken) - 1, -1, -1):
        g = n + 1 + step
        i = where.get(g)
        if i is None or now[i] != g:
            fail(f"gondola {g} is not on the circle when it should leave")
        now[i] = broken[step]
        where[broken[step]] = i
    s = (now[0] - 1) % n
    if any(now[i] != (i + s) % n + 1 for i in range(n)):
        fail("the replay does not start from a rotation of 1..n")


main()
