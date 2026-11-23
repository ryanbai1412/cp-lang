#!/usr/bin/env python3
# checker.py input output answer: the plan catches the thief (edge search on the tree, every
# city and road clear at the end), uses at most S detectives at once, T <= 20000 steps, and S
# equals the answer's S.
import sys
it = iter(open(sys.argv[1]).read().split())
n = int(next(it))
adj = {v: set() for v in range(1, n + 1)}
for _ in range(n - 1):
    u, v = int(next(it)), int(next(it)); adj[u].add(v); adj[v].add(u)
out = open(sys.argv[2]).read().split()
best = int(open(sys.argv[3]).read().split()[0])
S, T = int(out[0]), int(out[1])
if S != best: sys.exit(f"S = {S}, optimum {best}")
if T > 20000: sys.exit(f"T = {T} > 20000")
det = {v: 0 for v in adj}
dirty_v = set(adj); dirty_e = {frozenset((u, v)) for u in adj for v in adj[u]}
def spread():
    for e in dirty_e:
        for w in e:
            if det[w] == 0: dirty_v.add(w)
    stack = list(dirty_v)
    while stack:
        v = stack.pop()
        for w in adj[v]:
            e = frozenset((v, w))
            if e not in dirty_e: dirty_e.add(e)
            if det[w] == 0 and w not in dirty_v: dirty_v.add(w); stack.append(w)
pos, live = 2, 0
for _ in range(T):
    c = out[pos]; pos += 1
    if c == "L":
        x = int(out[pos]); pos += 1; det[x] += 1; live += 1; dirty_v.discard(x)
    elif c == "B":
        x = int(out[pos]); pos += 1
        if det[x] == 0: sys.exit(f"B {x}: nobody there")
        det[x] -= 1; live -= 1
    elif c == "M":
        x, y = int(out[pos]), int(out[pos + 1]); pos += 2
        if det[x] == 0 or y not in adj[x]: sys.exit(f"bad M {x} {y}")
        det[x] -= 1; det[y] += 1; dirty_v.discard(y); dirty_e.discard(frozenset((x, y)))
    else: sys.exit(f"bad step {c}")
    if live > S: sys.exit(f"{live} detectives out, S = {S}")
    for v in list(dirty_v):
        if det[v]: dirty_v.discard(v)
    spread()
if pos != len(out): sys.exit("trailing output")
if dirty_v or dirty_e: sys.exit(f"thief not caught: {len(dirty_v)} cities, {len(dirty_e)} roads dirty")
print("ok")
