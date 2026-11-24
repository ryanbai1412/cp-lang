#!/usr/bin/env python3
"""Checker for AAC4 P4: -1 exactly when the answer is -1; otherwise X in L..=R and an
n x m 0/1 matrix with X ones that leaves the baton in lane f."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m, s, f, lo, hi = inp[:6]
    k = inp[6:6 + m]
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if ref[0] == "-1" or out[:1] == ["-1"]:
        if out != ref:
            fail(f"got {' '.join(out[:1])}, want {ref[0]}")
        return
    if len(out) != 1 + n * m:
        fail(f"expected {1 + n * m} tokens, got {len(out)}")
    x = int(out[0])
    grid = [out[1 + i * m:1 + (i + 1) * m] for i in range(n)]
    if any(c not in ("0", "1") for row in grid for c in row):
        fail("matrix entries must be 0 or 1")
    if not lo <= x <= hi or sum(row.count("1") for row in grid) != x:
        fail(f"X = {x} is not the banana count in {lo}..={hi}")
    lane = s
    for j in range(m):
        if grid[lane - 1][j] == "1":
            lane = k[j]
    if lane != f:
        fail(f"baton ends in lane {lane}, want {f}")


if __name__ == "__main__":
    main()
