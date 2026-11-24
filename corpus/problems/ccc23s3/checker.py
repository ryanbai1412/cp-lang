#!/usr/bin/env python3
"""Checker for CCC 2023 S3: IMPOSSIBLE exactly when the answer says so, else an N x M grid of
lowercase letters with exactly R palindromic rows and C palindromic columns."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n, m, r, c = map(int, open(sys.argv[1]).read().split())
    out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[2:])
    if ref == ["IMPOSSIBLE"] or out == ["IMPOSSIBLE"]:
        if out != ref:
            fail(f"got {' '.join(out[:1])}, expected {' '.join(ref[:1])}")
        return
    if len(out) != n or any(len(w) != m or not w.isalpha() or not w.islower() for w in out):
        fail(f"expected {n} rows of {m} lowercase letters")
    rows = sum(w == w[::-1] for w in out)
    cols = sum((col := "".join(w[j] for w in out)) == col[::-1] for j in range(m))
    if (rows, cols) != (r, c):
        fail(f"{rows} palindromic rows and {cols} columns, expected {r} and {c}")


if __name__ == "__main__":
    main()
