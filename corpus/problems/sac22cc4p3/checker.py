#!/usr/bin/env python3
"""Checker for SAC '22 CC4 P3: any V minimizing sum |V - A_i| (every point between
the two middle values when N is even) is accepted."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    tok = open(sys.argv[1], encoding="utf-8").read().split()
    n, a = int(tok[0]), sorted(map(int, tok[1:]))
    out = open(sys.argv[2], encoding="utf-8").read().split()
    if len(out) != 1:
        fail(f"expected one value, got {len(out)}")
    try:
        v = int(out[0])
    except ValueError:
        fail(f"{out[0]!r} is not an integer")
    cost = lambda x: sum(abs(x - y) for y in a)
    if cost(v) != cost(a[n // 2]):
        fail(f"V = {v} costs {cost(v)}, the minimum is {cost(a[n // 2])}")


if __name__ == "__main__":
    main()
