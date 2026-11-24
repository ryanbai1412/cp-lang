#!/usr/bin/env python3
"""Checker for NOIP 1999 P3: `No Solution` exactly, else one cost within one rounding unit
(0.01) of the reference: inputs with one decimal can land on a .xx5 tie."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[2:])
    if ref == ["No", "Solution"] or out == ["No", "Solution"]:
        if out != ref:
            fail(f"got {' '.join(out)}, want {' '.join(ref)}")
        return
    if len(out) != len(ref):
        fail(f"expected {len(ref)} values, got {len(out)}")
    for g, w in zip(out, ref):
        try:
            ok = abs(float(g) - float(w)) <= 0.01 + 1e-9
        except ValueError:
            fail(f"{g} is not a number")
        if not ok:
            fail(f"{g} is too far from {w}")


if __name__ == "__main__":
    main()
