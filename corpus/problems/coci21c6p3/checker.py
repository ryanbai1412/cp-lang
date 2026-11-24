#!/usr/bin/env python3
# checker.py input output [answer]: -1 exactly when the reference says -1; otherwise at most
# 2e5 chargings "ball charge" after which every wire u v points at v (last charge at u is 1, at v is 0).
import sys
t = open(sys.argv[1]).read().split(); n, m = int(t[0]), int(t[1])
wires = [(int(t[2 + 2 * i]), int(t[3 + 2 * i])) for i in range(m)]
out = open(sys.argv[2]).read().split()
ans = open(sys.argv[3]).read().split() if len(sys.argv) > 3 else None
if ans is not None and (ans == ["-1"]) != (out == ["-1"]):
    sys.exit(1)
if out == ["-1"]:
    sys.exit(0)
k = int(out[0])
if not 0 <= k <= 200000 or len(out) != 1 + 2 * k:
    sys.exit(1)
last = {}
for j in range(k):
    b, c = int(out[1 + 2 * j]), int(out[2 + 2 * j])
    if not 1 <= b <= n or c not in (0, 1):
        sys.exit(1)
    last[b] = (j, c)
for u, v in wires:
    lu, lv = last.get(u), last.get(v)
    if lu is None and lv is None:
        sys.exit(1)
    if lv is None or (lu is not None and lu[0] > lv[0]):
        ok = lu[1] == 1
    else:
        ok = lv[1] == 0
    if not ok:
        sys.exit(1)
sys.exit(0)
