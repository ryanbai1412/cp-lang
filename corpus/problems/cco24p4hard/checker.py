#!/usr/bin/env python3
"""Checker for CCO 2024 Infiltration (hard): T <= 1440, every strategy row starts from its room
and moves only along corridors, Ondrej only at odd minutes and Edward only at even ones, every
pair of different rooms meets by time T, and S = max t(o, e) / d(o, e) is at most 15, the bound
for full marks."""
import sys
from collections import deque


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n = inp[0]
    adj = [set() for _ in range(n)]
    for k in range(n - 1):
        a, b = inp[1 + 2 * k], inp[2 + 2 * k]
        adj[a].add(b)
        adj[b].add(a)
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    if not out:
        fail("empty output")
    t = out[0]
    if not 1 <= t <= 1440:
        fail(f"T = {t} is out of range")
    if len(out) != 1 + 2 * n * t:
        fail("wrong number of tokens")
    rows = []
    for agent in range(2):
        for room in range(n):
            row = out[1 + (agent * n + room) * t:1 + (agent * n + room + 1) * t]
            prev = room
            for time, x in enumerate(row, 1):
                if not 0 <= x < n:
                    fail("room label out of range")
                if x != prev:
                    if time % 2 != (1 if agent == 0 else 0):
                        fail(f"agent {agent} moves at minute {time}, which is not its turn")
                    if x not in adj[prev]:
                        fail(f"agent {agent} from room {room} jumps from {prev} to {x}")
                prev = x
            rows.append(row)
    best = 0.0
    for e in range(n):
        dist = [-1] * n
        dist[e] = 0
        q = deque([e])
        while q:
            u = q.popleft()
            for v in adj[u]:
                if dist[v] < 0:
                    dist[v] = dist[u] + 1
                    q.append(v)
        ed = rows[n + e]
        for o in range(n):
            if o == e:
                continue
            met = next((i for i, (a, b) in enumerate(zip(rows[o], ed), 1) if a == b), 0)
            if not met:
                fail(f"rooms {o} and {e} never meet")
            best = max(best, met / dist[o])
    if best > 15:
        fail(f"S = {best:.4f} exceeds 15")
    sys.exit(0)


main()
