#!/usr/bin/env python3
# checker.py input output [answer]: at most 820000 moves, each from a non-empty tube to a
# different tube that is not full, after which every tube is empty or holds m balls of one colour.
import sys
tok = open(sys.argv[1]).read().split()
n, m = int(tok[0]), int(tok[1])
tube = [list(map(int, tok[2 + i * m:2 + (i + 1) * m])) for i in range(n)] + [[]]
out = list(map(int, open(sys.argv[2]).read().split()))
k = out[0]
if not 0 <= k <= 820000: sys.exit(f"{k} moves, limit 820000")
if len(out) != 1 + 2 * k: sys.exit(f"{(len(out) - 1) / 2} moves for k = {k}")
for i in range(k):
    x, y = out[1 + 2 * i], out[2 + 2 * i]
    if not (1 <= x <= n + 1 and 1 <= y <= n + 1) or x == y: sys.exit(f"move {i + 1}: bad tubes {x} {y}")
    if not tube[x - 1] or len(tube[y - 1]) == m: sys.exit(f"move {i + 1}: {x} empty or {y} full")
    tube[y - 1].append(tube[x - 1].pop())
for i, t in enumerate(tube):
    if t and (len(t) != m or len(set(t)) != 1): sys.exit(f"tube {i + 1} not sorted: {t[:10]}")
print("ok")
