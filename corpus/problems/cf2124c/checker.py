#!/usr/bin/env python3
"""Checker for CF 2124 C: any x in 1..1e9 such that dividing some b_i by x
leaves a chain a_1 | a_2 | ... | a_n. Each a_i is b_i or b_i / x, so a
two-state scan over the choices decides it."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def ok(b, x):
    prev = {1}
    for v in b:
        opts = {v} | ({v // x} if v % x == 0 else set())
        prev = {a for a in opts if any(a % p == 0 for p in prev)}
        if not prev:
            return False
    return True


def main():
    inp, out, _ = load()
    t, pi = int(inp[0]), 1
    if len(out) != t:
        fail(f"expected {t} values, got {len(out)}")
    for case in range(1, t + 1):
        n = int(inp[pi])
        b = [int(v) for v in inp[pi + 1:pi + 1 + n]]
        pi += 1 + n
        x = int(out[case - 1])
        if not 1 <= x <= 10**9:
            fail(f"case {case}: x = {x} out of range")
        if not ok(b, x):
            fail(f"case {case}: no beautiful a gives b with x = {x}")


if __name__ == "__main__":
    main()
