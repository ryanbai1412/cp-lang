#!/usr/bin/env python3
"""Checker for DMOPC '20 Contest 4 P5: 0 exactly when the answer says it is impossible, else N
values in {1, -1} whose cyclic length-K window products sum to 0."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n, k = map(int, open(sys.argv[1]).read().split())
    out = open(sys.argv[2]).read().split()
    possible = open(sys.argv[3]).read().split() != ["0"]
    if out == ["0"]:
        if possible:
            fail("printed 0 but a message exists")
        return
    if not possible:
        fail("printed a message but none exists")
    if len(out) != n or any(t not in ("1", "-1") for t in out):
        fail(f"expected {n} values in {{1, -1}}")
    neg = [t == "-1" for t in out]
    pre = [0]
    for x in neg + neg:
        pre.append(pre[-1] ^ x)
    total = sum(-1 if pre[i + k] ^ pre[i] else 1 for i in range(n))
    if total:
        fail(f"window products sum to {total}")


main()
