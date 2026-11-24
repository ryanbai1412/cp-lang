#!/usr/bin/env python3
"""Checker for IOI 2018 Doll: the circuit (C[0..=M], then X and Y, each led by its length)
plays the trigger sequence A exactly, returns to the origin with every switch back on X, and
uses at most n + ceil(log2 n) switches (full score)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def circuit(text, m):
    lines = text.strip().split("\n")
    if len(lines) != 3:
        fail(f"expected 3 lines, got {len(lines)}")
    try:
        c, x, y = ([int(t) for t in line.split()] for line in lines)
    except ValueError:
        fail("output is not integers")
    if len(c) != m + 1 or not x or not y or x[0] != len(x) - 1 or y[0] != len(y) - 1 or x[0] != y[0]:
        fail("C has the wrong length, or X and Y disagree with their counts")
    return c, x[1:], y[1:]


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    m, n = inp[:2]
    a = inp[2:2 + n]
    c, x, y = circuit(open(sys.argv[2]).read(), m)
    s = len(x)
    if s > n + (n - 1).bit_length():
        fail(f"{s} switches, more than n + ceil(log2 n) = {n + (n - 1).bit_length()}")
    if any(not -s <= e <= m for e in c + x + y):
        fail("an exit names no trigger or switch")
    on_y = [False] * s
    seen = []
    at = c[0]
    for _ in range(20 * (n + s) + 10):
        if at == 0:
            break
        if at > 0:
            seen.append(at)
            if len(seen) > n:
                fail("the ball plays more triggers than A")
            at = c[at]
        else:
            k = -at - 1
            at = y[k] if on_y[k] else x[k]
            on_y[k] = not on_y[k]
    else:
        fail("the ball never returns to the origin")
    if seen != a:
        fail("the triggers played differ from A")
    if any(on_y):
        fail("a switch is left on Y")


main()
