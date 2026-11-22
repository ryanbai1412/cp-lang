#!/usr/bin/env python3
"""Checker for CF 2138 E1: n <= 80, entries in {-1,0,1}, <=3 non-zeros per row/col, det == x."""
import sys


def exact_det(mat):
    n = len(mat)
    a = [row[:] for row in mat]
    sign = 1
    prev = 1
    for k in range(n):
        piv = -1
        for i in range(k, n):
            if a[i][k] != 0:
                piv = i
                break
        if piv == -1:
            return 0
        if piv != k:
            a[k], a[piv] = a[piv], a[k]
            sign = -sign
        for i in range(k + 1, n):
            for j in range(k + 1, n):
                a[i][j] = (a[i][j] * a[k][k] - a[i][k] * a[k][j]) // prev
        prev = a[k][k]
    return sign * a[n - 1][n - 1]


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: cf2138e1.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        inp = [int(v) for v in f.read().split()]
    with open(sys.argv[2], encoding="utf-8") as f:
        out = [int(v) for v in f.read().split()]

    t = inp[0]
    pos = 0
    for tc in range(t):
        x = inp[1 + tc]
        if pos >= len(out):
            sys.stderr.write(f"case {tc + 1}: unexpected EOF\n")
            sys.exit(1)
        n = out[pos]
        pos += 1
        if not (1 <= n <= 80):
            sys.stderr.write(f"case {tc + 1}: n={n} out of [1, 80]\n")
            sys.exit(1)
        if pos + n * n > len(out):
            sys.stderr.write(f"case {tc + 1}: truncated matrix\n")
            sys.exit(1)
        mat = []
        for i in range(n):
            row = out[pos : pos + n]
            pos += n
            for v in row:
                if v not in (-1, 0, 1):
                    sys.stderr.write(f"case {tc + 1}: bad entry {v}\n")
                    sys.exit(1)
            if sum(1 for v in row if v != 0) > 3:
                sys.stderr.write(f"case {tc + 1}: row {i} has >3 non-zeros\n")
                sys.exit(1)
            mat.append(row)
        for j in range(n):
            if sum(1 for i in range(n) if mat[i][j] != 0) > 3:
                sys.stderr.write(f"case {tc + 1}: col {j} has >3 non-zeros\n")
                sys.exit(1)
        d = exact_det(mat)
        if d != x:
            sys.stderr.write(f"case {tc + 1}: det={d} != expected {x}\n")
            sys.exit(1)
    if pos != len(out):
        sys.stderr.write(f"extra trailing tokens: {len(out) - pos}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
