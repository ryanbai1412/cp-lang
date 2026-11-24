#!/usr/bin/env python3
"""Checker for NOI 2020 Dish: per test, -1 exactly when the answer is -1; otherwise m lines
"i x" or "i x j y" (positive grams, two different ingredients, x (+ y) = k) that use every
ingredient up exactly."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = iter(open(sys.argv[1]).read().split())
    out = [l.split() for l in open(sys.argv[2]).read().splitlines() if l.strip()]
    ref = [l.split() for l in open(sys.argv[3]).read().splitlines() if l.strip()]
    oi = ri = 0
    for t in range(int(next(inp))):
        n, m, k = int(next(inp)), int(next(inp)), int(next(inp))
        d = [int(next(inp)) for _ in range(n)]
        if ri >= len(ref) or oi >= len(out):
            fail(f"test {t + 1}: output ends early")
        if ref[ri] == ["-1"]:
            if out[oi] != ["-1"]:
                fail(f"test {t + 1}: expected -1")
            ri += 1; oi += 1
            continue
        ri += m
        if out[oi] == ["-1"]:
            fail(f"test {t + 1}: printed -1 but a plan exists")
        left = d[:]
        for dish in out[oi:oi + m]:
            v = [int(x) for x in dish]
            if len(v) not in (2, 4):
                fail(f"test {t + 1}: bad dish {dish}")
            pairs = list(zip(v[::2], v[1::2]))
            if sum(g for _, g in pairs) != k or any(g <= 0 or not 1 <= i <= n for i, g in pairs):
                fail(f"test {t + 1}: bad dish {dish}")
            if len(pairs) == 2 and pairs[0][0] == pairs[1][0]:
                fail(f"test {t + 1}: one ingredient twice in {dish}")
            for i, g in pairs:
                left[i - 1] -= g
        if len(out) < oi + m or any(left):
            fail(f"test {t + 1}: ingredients not used up exactly")
        oi += m
    if oi != len(out):
        fail("extra output")


if __name__ == "__main__":
    main()
