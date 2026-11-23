#!/usr/bin/env python3
"""Checker for CF 2219 C: one real per test case, accepted when
|got - want| / max(1, |want|) <= 1e-6, as the statement states."""
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
    for case, (g, w) in enumerate(zip(out, ref), 1):
        got, want = float(g), float(w)
        if not abs(got - want) / max(1.0, abs(want)) <= 1e-6:
            fail(f"case {case}: {g} is too far from {w}")


if __name__ == "__main__":
    main()
