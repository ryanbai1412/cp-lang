#!/usr/bin/env python3
"""Checker for CF 2062 G: verify swap sequence transforms p into q with optimal cost."""
import sys


def parse_case_ops(tokens, pos, n, p, q, label):
    if pos >= len(tokens):
        sys.stderr.write(f"{label}: unexpected EOF reading k\n")
        sys.exit(1)
    k = tokens[pos]
    pos += 1
    if k < 0 or k > n * n:
        sys.stderr.write(f"{label}: invalid k={k} for n={n}\n")
        sys.exit(1)
    if pos + 2 * k > len(tokens):
        sys.stderr.write(f"{label}: unexpected EOF reading {k} swaps\n")
        sys.exit(1)
    cur = list(p)
    cost = 0
    for _ in range(k):
        i = tokens[pos]
        j = tokens[pos + 1]
        pos += 2
        if not (1 <= i <= n and 1 <= j <= n and i != j):
            sys.stderr.write(f"{label}: invalid swap ({i}, {j}) for n={n}\n")
            sys.exit(1)
        cost += min(abs(i - j), abs(cur[i - 1] - cur[j - 1]))
        cur[i - 1], cur[j - 1] = cur[j - 1], cur[i - 1]
    if cur != q:
        sys.stderr.write(f"{label}: final perm {cur} != target {q}\n")
        sys.exit(1)
    return pos, cost


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: cf2062g.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        inp = [int(x) for x in f.read().split()]
    with open(sys.argv[2], encoding="utf-8") as f:
        out = [int(x) for x in f.read().split()]
    with open(sys.argv[3], encoding="utf-8") as f:
        ans = [int(x) for x in f.read().split()]
    t = inp[0]
    ip = 1
    op = 0
    ap = 0
    for tc in range(1, t + 1):
        n = inp[ip]
        p = inp[ip + 1 : ip + 1 + n]
        q = inp[ip + 1 + n : ip + 1 + 2 * n]
        ip += 1 + 2 * n
        op, out_cost = parse_case_ops(out, op, n, p, q, f"case {tc} out")
        ap, ans_cost = parse_case_ops(ans, ap, n, p, q, f"case {tc} ref")
        if out_cost != ans_cost:
            sys.stderr.write(
                f"case {tc}: cost {out_cost} != optimal {ans_cost}\n"
            )
            sys.exit(1)
    if op != len(out):
        sys.stderr.write(f"extra tokens in output: {len(out) - op}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
