#!/usr/bin/env python3
"""Checker for AtCoder DP J: one expectation, accepted when
|got - want| / max(1, |want|) <= 1e-9, the error the statement allows."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[2:])
    if len(out) != len(ref):
        fail(f"expected {len(ref)} values, got {len(out)}")
    for g, w in zip(out, ref):
        if not abs(float(g) - float(w)) / max(1.0, abs(float(w))) <= 1e-9:
            fail(f"{g} is too far from {w}")


if __name__ == "__main__":
    main()
