#!/usr/bin/env python3
# checker.py input output answer: NO only when the reference says NO; otherwise N - 1 tubes between
# distinct buildings, every existing tube among them, no banned one, each building but one sending
# to exactly one other, and all garbage reaching that one root. Such a tree is right even where the
# reference says NO: submission 5678057 misses some (see meta.json).
import sys
tok = list(map(int, open(sys.argv[1]).read().split()))
n, m, k = tok[:3]
pairs = [(tok[3 + 2 * i], tok[4 + 2 * i]) for i in range(m + k)]
must, ban = pairs[:m], set(pairs[m:])
out = open(sys.argv[2]).read().split()
ref_no = open(sys.argv[3]).read().split() == ["NO"]
if out == ["NO"]:
    sys.exit(0 if ref_no else "NO, but the reference builds a tree")
if len(out) != 2 * (n - 1): sys.exit(f"{len(out)} numbers, want {2 * (n - 1)}")
arcs = [(int(out[2 * i]), int(out[2 * i + 1])) for i in range(n - 1)]
up = {}
for a, b in arcs:
    if not (0 <= a < n and 0 <= b < n and a != b): sys.exit(f"bad tube {a} {b}")
    if (a, b) in ban: sys.exit(f"banned tube {a} {b}")
    if a in up: sys.exit(f"{a} sends to {up[a]} and {b}")
    up[a] = b
for a, b in must:
    if up.get(a) != b: sys.exit(f"existing tube {a} {b} missing")
state = [0] * n
for s in range(n):
    path = []
    v = s
    while state[v] == 0 and v in up:
        state[v] = 1; path.append(v); v = up[v]
    if state[v] == 1: sys.exit(f"cycle through {v}")
    for x in path: state[x] = 2
    state[v] = 2
print("ok")
