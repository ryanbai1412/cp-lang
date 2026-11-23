#!/usr/bin/env python3
"""Judge for EGOI 2022 "Social Engineering": `interactor.py CASE [--log-in F] [--log-out F] -- SOLUTION...`.

CASE is `n m`, m lines `u v`, then Maria's seed (0: she takes her smallest free friend, else
a seeded random one). Sends `n m` and the edges. `?` asks Maria's move from vertex 1: a free
neighbour, or 0 when she has none (she resigns). An integer is the solution's move along a
free edge. `!` is the function returning: right only before any `?` when Maria has a winning
strategy (some component without vertex 1 holds an odd number of its neighbours), else only
after Maria resigned."""
import os, random, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "lib"))
from interactor_lib import run_interactor


def maria_wins(n, edges):
    up = list(range(n + 1))
    def find(x):
        while up[x] != x:
            up[x] = up[up[x]]
            x = up[x]
        return x
    for u, v in edges:
        if u != 1 and v != 1:
            up[find(u)] = find(v)
    odd = {}
    for u, v in edges:
        if 1 in (u, v):
            r = find(u + v - 1)
            odd[r] = not odd.get(r, False)
    return any(odd.values())


def drive(case_text, io):
    t = list(map(int, case_text.split()))
    n, m = t[0], t[1]
    edges = [(t[2 + 2 * i], t[3 + 2 * i]) for i in range(m)]
    seed = t[2 + 2 * m]
    rng = random.Random(seed)
    free = [set() for _ in range(n + 1)]
    for u, v in edges:
        free[u].add(v)
        free[v].add(u)
    io.send(f"{n} {m}\n" + "\n".join(f"{u} {v}" for u, v in edges))
    lost = maria_wins(n, edges)
    at, asked, resigned = 1, 0, False
    while True:
        w = io.recv_line().split()
        if w == ["!"]:
            if asked == 0 and not lost:
                raise RuntimeError("returned at once, but Maria has no winning strategy")
            if asked > 0 and not resigned:
                raise RuntimeError("returned while Maria can still move")
            break
        if resigned:
            raise RuntimeError(f"{' '.join(w)[:40]!r} after Maria resigned")
        if w == ["?"]:
            if lost:
                raise RuntimeError("played on though Maria has a winning strategy")
            if at != 1:
                raise RuntimeError(f"asked Maria to move from {at}")
            asked += 1
            if not free[1]:
                resigned = True
                io.send("0")
                continue
            opts = sorted(free[1])
            v = opts[0] if seed == 0 else rng.choice(opts)
        elif len(w) == 1 and w[0].lstrip("-").isdigit():
            v = int(w[0])
            if at == 1:
                raise RuntimeError(f"moved to {v} on Maria's turn")
            if v not in free[at]:
                raise RuntimeError(f"no free edge {at} {v}")
        else:
            raise RuntimeError(f"bad line {' '.join(w)[:40]!r}")
        free[at].discard(v)
        free[v].discard(at)
        if w == ["?"]:
            io.send(str(v))
        at = v
    sys.stderr.write(f"ok: {'resigned' if asked == 0 else f'won after {asked - 1} Maria moves'}\n")


run_interactor(drive)
