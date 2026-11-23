#!/usr/bin/env python3
# checker.py input output [answer]: for each start i, a route of length n (the minimum: every
# building once) beginning at i, visiting all buildings, with at most one colour switch.
import sys
t = open(sys.argv[1]).read().split()
n = int(t[0]); rows = t[1:]
def red(a, b):
    if a < b: a, b = b, a
    return rows[a - 1][b] == 'R'
out = list(map(int, open(sys.argv[2]).read().split()))
pos = 0
for i in range(n):
    m = out[pos]; route = [v - 1 for v in out[pos + 1:pos + 1 + m]]; pos += 1 + m
    if m != n: sys.exit(f"start {i + 1}: length {m}, minimum {n}")
    if route[0] != i or sorted(route) != list(range(n)): sys.exit(f"start {i + 1}: not a route from it over every building")
    cols = [red(route[j], route[j + 1]) for j in range(n - 1)]
    if sum(cols[j] != cols[j + 1] for j in range(n - 2)) > 1: sys.exit(f"start {i + 1}: more than one switch")
if pos != len(out): sys.exit("trailing output")
print("ok")
