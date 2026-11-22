#!/usr/bin/env python3
"""Checker for CF 2187 B: any non-negative (p, q) with p & q == 0 matching orig's |p-x| + |q-y|."""
import sys


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: cf2187b.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        inp = [int(x) for x in f.read().split()]
    with open(sys.argv[2], encoding="utf-8") as f:
        out = [int(x) for x in f.read().split()]
    with open(sys.argv[3], encoding="utf-8") as f:
        ans = [int(x) for x in f.read().split()]
    t = inp[0]
    if len(out) != 2 * t:
        sys.stderr.write(f"expected {2 * t} integers, got {len(out)}\n")
        sys.exit(1)
    if len(ans) != 2 * t:
        sys.stderr.write(f"answer has {len(ans)} integers, expected {2 * t}\n")
        sys.exit(1)
    for i in range(t):
        x, y = inp[1 + 2 * i], inp[2 + 2 * i]
        p, q = out[2 * i], out[2 * i + 1]
        ap, aq = ans[2 * i], ans[2 * i + 1]
        if p < 0 or q < 0:
            sys.stderr.write(f"case {i + 1}: negative output ({p}, {q})\n")
            sys.exit(1)
        if (p & q) != 0:
            sys.stderr.write(f"case {i + 1}: p & q = {p & q} != 0 for ({p}, {q})\n")
            sys.exit(1)
        d_out = abs(p - x) + abs(q - y)
        d_ans = abs(ap - x) + abs(aq - y)
        if d_out != d_ans:
            sys.stderr.write(f"case {i + 1} ({x}, {y}): dist {d_out} != optimal {d_ans}\n")
            sys.exit(1)


if __name__ == "__main__":
    main()
