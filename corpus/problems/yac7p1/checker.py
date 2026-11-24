#!/usr/bin/env python3
"""Checker for yac7p1: the first line is the least total of page turns (from the answer),
the second a permutation of 1..N whose layout has exactly that total. A piece of a pages
starting on page p turns once per odd page in [p, p + a - 2]."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def odd_upto(x):
    return (x + 1) // 2 if x > 0 else 0


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    tok = open(sys.argv[1], encoding="utf-8").read().split()
    n, a = int(tok[0]), list(map(int, tok[1:]))
    want = int(open(sys.argv[3], encoding="utf-8").read().split()[0])
    out = open(sys.argv[2], encoding="utf-8").read().split()
    if len(out) != n + 1:
        fail(f"expected {n + 1} numbers, got {len(out)}")
    try:
        got, order = int(out[0]), list(map(int, out[1:]))
    except ValueError:
        fail("non-integer token")
    if got != want:
        fail(f"total {got}, the least is {want}")
    if sorted(order) != list(range(1, n + 1)):
        fail("the order is not a permutation of 1..N")
    page, total = 1, 0
    for i in order:
        last = page + a[i - 1] - 2
        total += odd_upto(last) - odd_upto(page - 1)
        page += a[i - 1]
    if total != want:
        fail(f"the printed order turns {total} pages, not {want}")


if __name__ == "__main__":
    main()
