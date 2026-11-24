#!/usr/bin/env python3
# checker.py input output answer: line 1 is a schedule (a permutation of the planes, plane i in
# a slot <= k_i, every rule a -> b with a first); line 2, the earliest slots, equals the answer's.
import sys
tok = open(sys.argv[1]).read().split()
n, m = int(tok[0]), int(tok[1])
k = [0] + [int(x) for x in tok[2:2 + n]]
rules = [(int(tok[2 + n + 2 * i]), int(tok[3 + n + 2 * i])) for i in range(m)]
out = open(sys.argv[2]).read().split("\n")
ans = open(sys.argv[3]).read().split("\n")
order = [int(x) for x in out[0].split()]
if sorted(order) != list(range(1, n + 1)): sys.exit("line 1 is not a permutation of the planes")
slot = {p: i + 1 for i, p in enumerate(order)}
for p in range(1, n + 1):
    if slot[p] > k[p]: sys.exit(f"plane {p} in slot {slot[p]} > k = {k[p]}")
for a, b in rules:
    if slot[a] > slot[b]: sys.exit(f"rule {a} -> {b} broken")
if out[1].split() != ans[1].split(): sys.exit("line 2 (earliest slots) differs")
