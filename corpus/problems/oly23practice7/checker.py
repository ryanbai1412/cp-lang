#!/usr/bin/env python3
"""Checker for DMOJ oly23practice7: the verdict (YES/NO) must match the reference's; a YES must
give a permutation of 1..m within every [l_i, r_i] under which Kruskal picks exactly edges
1..n-1 (weights are distinct, so the MST is unique)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = iter(map(int, open(sys.argv[1]).read().split()))
    out = open(sys.argv[2]).read().split()
    ans = open(sys.argv[3]).read().split()
    op = ap = 0
    for case in range(1, next(inp) + 1):
        n, m = next(inp), next(inp)
        edges = [(next(inp) - 1, next(inp) - 1, next(inp), next(inp)) for _ in range(m)]
        if ap >= len(ans) or op >= len(out):
            fail(f"case {case}: output ended early")
        want, got = ans[ap], out[op]
        ap += 1 + (m if want == "YES" else 0)
        op += 1
        if got != want:
            fail(f"case {case}: expected {want}, got {got}")
        if got == "NO":
            continue
        try:
            w = [int(x) for x in out[op:op + m]]
        except ValueError:
            fail(f"case {case}: weights are not integers")
        op += m
        if len(w) != m or sorted(w) != list(range(1, m + 1)):
            fail(f"case {case}: weights are not a permutation of 1..{m}")
        for i, (a, b, lo, hi) in enumerate(edges):
            if not lo <= w[i] <= hi:
                fail(f"case {case}: edge {i + 1} weight {w[i]} outside [{lo}, {hi}]")
        par = list(range(n))

        def find(x):
            while par[x] != x:
                par[x] = par[par[x]]
                x = par[x]
            return x

        for i in sorted(range(m), key=w.__getitem__):
            a, b = find(edges[i][0]), find(edges[i][1])
            if a != b:
                if i >= n - 1:
                    fail(f"case {case}: non-tree edge {i + 1} joins the MST")
                par[a] = b
    if op != len(out):
        fail("extra output")


if __name__ == "__main__":
    main()
