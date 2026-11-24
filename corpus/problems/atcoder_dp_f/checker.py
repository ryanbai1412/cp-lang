#!/usr/bin/env python3
"""Checker for AtCoder DP F: the output must be a common subsequence of s and t
as long as the answer's (any longest one is accepted)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def subseq(x, s):
    it = iter(s)
    return all(c in it for c in x)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    s, t = open(sys.argv[1], encoding="utf-8").read().split()
    got = "".join(open(sys.argv[2], encoding="utf-8").read().split())
    want = "".join(open(sys.argv[3], encoding="utf-8").read().split())
    if len(got) != len(want):
        fail(f"length {len(got)}, want {len(want)}")
    if not (subseq(got, s) and subseq(got, t)):
        fail(f"{got!r} is not a common subsequence")


if __name__ == "__main__":
    main()
