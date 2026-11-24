#!/usr/bin/env python3
"""Checker for TLE '17 C5 P6: a string of n characters over 0, 1 and x, fixing as many inputs as
the reference, under which the output gate still gives the unfixed circuit's value for x = 0 and
x = 1."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m = inp[0], inp[1]
    gates = inp[2:2 + 2 * m]
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if len(out) != 1 or len(out[0]) != n or set(out[0]) - set("01x"):
        fail(f"want one string of {n} characters over 0, 1, x")
    if out[0].count("x") != ref[0].count("x"):
        fail(f"{n - out[0].count('x')} inputs fixed, want {n - ref[0].count('x')}")

    def run(inputs):
        val = inputs + [False] * m
        for i in range(m):
            val[n + i] = not (val[gates[2 * i] - 1] and val[gates[2 * i + 1] - 1])
        return val[-1]

    for x in (False, True):
        fixed = [x if c == "x" else c == "1" for c in out[0]]
        if run(fixed) != run([x] * n):
            fail(f"output differs from the unfixed circuit at x = {int(x)}")


if __name__ == "__main__":
    main()
