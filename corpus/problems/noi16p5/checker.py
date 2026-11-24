#!/usr/bin/env python3
"""Checker for NOI 2016 P5: one non-negative decimal, at most 2p places, within 10^-p of the
reference (p is the input's third number)."""
import re, sys
from decimal import Decimal, getcontext


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    p = int(open(sys.argv[1]).read().split()[2])
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if len(out) != 1:
        fail(f"expected one number, got {len(out)} tokens")
    m = re.fullmatch(r"\d+(\.(\d+))?", out[0])
    if not m:
        fail("not a non-negative decimal")
    if m.group(2) and len(m.group(2)) > 2 * p:
        fail(f"more than {2 * p} decimal places")
    getcontext().prec = 4 * p + 20
    if abs(Decimal(out[0]) - Decimal(ref[0])) >= Decimal(10) ** -p:
        fail(f"off by 10^-{p} or more")


main()
