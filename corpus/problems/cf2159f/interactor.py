#!/usr/bin/env python3
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


def drive(case_text, io):
    toks = iter(case_text.split())
    t = int(next(toks))
    io.send(str(t))
    for _ in range(t):
        n = int(next(toks))
        m = int(next(toks))
        grid = [[int(next(toks)) for _ in range(n)] for _ in range(n)]
        moves_list = [next(toks) for _ in range(n)]

        # Precompute f(l, t) for l in 1..n, t in 1..(2n-1)
        vals = {}
        all_vals = []
        for l in range(1, n + 1):
            # Initial snake body cells from tail to head: (0, l-1), ..., (0, 0)
            seq = [grid[0][c] for c in range(l - 1, -1, -1)]
            r = c = 0
            for ch in moves_list[l - 1]:
                if ch == "D":
                    r += 1
                else:
                    c += 1
                seq.append(grid[r][c])
            for step in range(2 * n - 1):
                v = max(seq[step : step + l])
                vals[(l, step + 1)] = v
                all_vals.append(v)
        expected = sorted(all_vals)[:m]

        io.send(f"{n} {m}")
        for row in grid:
            io.send(" ".join(map(str, row)))

        queries = 0
        limit = 120 * n + m
        while True:
            parts = io.recv_line().split()
            if parts[0] == "?":
                l, step = int(parts[1]), int(parts[2])
                if not (1 <= l <= n and 1 <= step <= 2 * n - 1):
                    raise RuntimeError(f"bad query {l} {step}")
                queries += 1
                if queries > limit:
                    raise RuntimeError("too many queries")
                io.send(str(vals[(l, step)]))
            elif parts[0] == "!":
                got = [int(x) for x in parts[1:]]
                if got != expected:
                    raise RuntimeError(f"wrong answer {got}, expected {expected}")
                break
            else:
                raise RuntimeError(f"bad command {parts}")


run_interactor(drive)
