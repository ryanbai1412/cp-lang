#!/usr/bin/env python3
"""Checker for CCO 2021 P2: per query IMPOSSIBLE exactly when the answer is IMPOSSIBLE; otherwise
at least one digit, each one of the given digits, whose base-K value is the query."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    k, q, d = inp[:3]
    digits = set(inp[4:4 + d])
    queries = inp[4 + d:4 + d + q]
    out = open(sys.argv[2]).read().split("\n")
    ref = open(sys.argv[3]).read().split("\n")
    for i, x in enumerate(queries):
        got = out[i].split() if i < len(out) else []
        want = ref[i].split() if i < len(ref) else []
        if want == ["IMPOSSIBLE"] or got == ["IMPOSSIBLE"]:
            if got != want:
                fail(f"query {i + 1}: got {' '.join(got[:3])}, want {' '.join(want[:3])}")
            continue
        if not got:
            fail(f"query {i + 1}: no digits")
        value = 0
        for t in got:
            b = int(t)
            if b not in digits:
                fail(f"query {i + 1}: {b} is not a digit")
            value = value * k + b
        if value != x:
            fail(f"query {i + 1}: the digits spell {value}, not {x}")


main()
