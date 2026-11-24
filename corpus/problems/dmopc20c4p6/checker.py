#!/usr/bin/env python3
"""Checker for DMOPC '20 Contest 4 P6: A equals the answer's (always N), and with the printed
opinions every child has an even number of friends sharing its opinion."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = open(sys.argv[1]).read().split()
    n, m = int(inp[0]), int(inp[1])
    out = open(sys.argv[2]).read().split()
    want = int(open(sys.argv[3]).read().split()[0])
    if len(out) != n + 1 or not out[0].isdigit() or any(t not in ("1", "2") for t in out[1:]):
        fail(f"expected A and {n} opinions in {{1, 2}}")
    if int(out[0]) != want:
        fail(f"A = {out[0]}, want {want}")
    col = out[1:]
    same = [0] * n
    for i in range(m):
        a, b = int(inp[2 + 2 * i]) - 1, int(inp[3 + 2 * i]) - 1
        if col[a] == col[b]:
            same[a] ^= 1
            same[b] ^= 1
    happy = n - sum(same)
    if happy != want:
        fail(f"{happy} children pair up, A claims {want}")


main()
