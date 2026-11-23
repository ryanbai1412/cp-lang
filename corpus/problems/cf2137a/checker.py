#!/usr/bin/env python3
"""Checker for CF 2137 A: any positive start that reaches x after k Collatz steps."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def main():
    inp, out, _ = load()
    t = int(inp[0])
    if len(out) != t:
        fail(f"expected {t} values, got {len(out)}")
    for i in range(t):
        k, x = int(inp[1 + 2 * i]), int(inp[2 + 2 * i])
        y = start = int(out[i])
        if start < 1:
            fail(f"case {i + 1}: {start} is not positive")
        for _ in range(k):
            y = y // 2 if y % 2 == 0 else 3 * y + 1
        if y != x:
            fail(f"case {i + 1}: {start} reaches {y}, not {x}, after {k} steps")


if __name__ == "__main__":
    main()
