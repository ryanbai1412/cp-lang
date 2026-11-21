#!/usr/bin/env python3
"""Checker for CF 2150 F (Cycle Closing): at most 2 operations of simple paths
of fixed length k >= 3 using pre-operation edges, adding (v_1, v_k), ending in K_n."""
import sys


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: cf2150f.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = [int(x) for x in open(sys.argv[1], encoding="utf-8").read().split()]
    out = [int(x) for x in open(sys.argv[2], encoding="utf-8").read().split()]
    it_in = iter(inp)
    it_out = iter(out)
    t = next(it_in)
    for tc in range(1, t + 1):
        n = next(it_in)
        m = next(it_in)
        adj = [set() for _ in range(n + 1)]
        for _ in range(m):
            u, v = next(it_in), next(it_in)
            adj[u].add(v)
            adj[v].add(u)
        try:
            q = next(it_out)
        except StopIteration:
            sys.stderr.write(f"case {tc}: unexpected EOF reading q\n")
            sys.exit(1)
        if not (0 <= q <= 2):
            sys.stderr.write(f"case {tc}: q={q} not in 0..=2\n")
            sys.exit(1)
        for op in range(1, q + 1):
            try:
                k = next(it_out)
                cnt = next(it_out)
            except StopIteration:
                sys.stderr.write(f"case {tc} op {op}: unexpected EOF\n")
                sys.exit(1)
            if not (2 <= k <= n):
                sys.stderr.write(f"case {tc} op {op}: k={k} not in 2..={n}\n")
                sys.exit(1)
            if cnt < 0 or cnt > n * (n - 1) // 2:
                sys.stderr.write(f"case {tc} op {op}: bad cnt={cnt}\n")
                sys.exit(1)
            to_add = []
            for p_idx in range(cnt):
                try:
                    path = [next(it_out) for _ in range(k)]
                except StopIteration:
                    sys.stderr.write(f"case {tc} op {op} path {p_idx}: unexpected EOF\n")
                    sys.exit(1)
                if len(set(path)) != k:
                    sys.stderr.write(f"case {tc} op {op} path {p_idx}: non-simple path {path}\n")
                    sys.exit(1)
                for v in path:
                    if not (1 <= v <= n):
                        sys.stderr.write(f"case {tc} op {op} path {p_idx}: vertex {v} out of range\n")
                        sys.exit(1)
                for a, b in zip(path, path[1:]):
                    if b not in adj[a]:
                        sys.stderr.write(f"case {tc} op {op} path {p_idx}: missing edge ({a}, {b})\n")
                        sys.exit(1)
                to_add.append((path[0], path[-1]))
            for a, b in to_add:
                adj[a].add(b)
                adj[b].add(a)
        for u in range(1, n + 1):
            if len(adj[u]) != n - 1:
                sys.stderr.write(f"case {tc}: graph not complete, deg({u})={len(adj[u])} != {n-1}\n")
                sys.exit(1)
    try:
        extra = next(it_out)
        sys.stderr.write(f"trailing output token: {extra}\n")
        sys.exit(1)
    except StopIteration:
        pass


if __name__ == "__main__":
    main()
