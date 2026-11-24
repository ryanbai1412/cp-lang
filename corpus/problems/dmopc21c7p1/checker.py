#!/usr/bin/env python3
# checker.py input output answer: -1 exactly when the answer is -1; otherwise an N x M grid that
# keeps the given cells, holds values in 1..=1e18, and increases strictly along rows and columns.
import sys
t = list(map(int, open(sys.argv[1]).read().split()))
n, m, a = t[0], t[1], t[2:]
ans = open(sys.argv[3]).read().split()
try:
    out = list(map(int, open(sys.argv[2]).read().split()))
except ValueError:
    sys.exit("output is not integers")
if ans == ["-1"]:
    sys.exit(0 if out == [-1] else "expected -1")
if out == [-1]:
    sys.exit("printed -1 but a filling exists")
if len(out) != n * m:
    sys.exit(f"expected {n * m} values, got {len(out)}")
for i in range(n * m):
    if a[i] and out[i] != a[i]:
        sys.exit(f"cell {i // m + 1} {i % m + 1} changed")
    if not 1 <= out[i] <= 10**18:
        sys.exit(f"cell {i // m + 1} {i % m + 1} out of range")
    if i % m and out[i - 1] >= out[i]:
        sys.exit(f"row {i // m + 1} not increasing")
    if i >= m and out[i - m] >= out[i]:
        sys.exit(f"column {i % m + 1} not increasing")
