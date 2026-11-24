#!/usr/bin/env python3
"""Checker for IOI 2009 Hiring: any best set is accepted. The hired candidates must be
distinct, as many as the reference's, and cost no more than W and than the reference's set,
where a set costs (max S/Q over it) * (sum of its Q), compared exactly."""
import sys
from fractions import Fraction


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def cost(ask, hired):
    if not hired:
        return Fraction(0)
    return max(Fraction(ask[k][0], ask[k][1]) for k in hired) * sum(ask[k][1] for k in hired)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, w = inp[0], inp[1]
    ask = {k + 1: (inp[2 + 2 * k], inp[3 + 2 * k]) for k in range(n)}
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    ref = list(map(int, open(sys.argv[3]).read().split()))
    if not out or out[0] != ref[0]:
        fail(f"hires {out[:1]}, want {ref[0]}")
    hired = out[1:]
    if len(hired) != out[0] or len(set(hired)) != len(hired) or any(k not in ask for k in hired):
        fail("the hired list is not that many distinct candidates")
    mine, best = cost(ask, hired), cost(ask, ref[1:])
    if mine > w:
        fail(f"costs {float(mine)}, over the budget {w}")
    if mine != best:
        fail(f"costs {float(mine)}, want {float(best)}")


main()
