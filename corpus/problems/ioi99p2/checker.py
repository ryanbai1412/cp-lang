#!/usr/bin/env python3
# checker.py input output [answer]: the claimed sum is the optimum (computed here, since the
# accepted C++ allows spans of 1001), and the listed items are non-overlapping covering sequences
# of at most 1000 letters whose word lengths add up to it. The answer file is not read.
import sys
SPAN = 1000
tok = open(sys.argv[1]).read().split()
m = int(tok[0]); words = tok[1:1 + m]; s = tok[1 + m]; n = len(s)
by = {}
for i, w in enumerate(words):
    for j in range(len(w) - 1, -1, -1):
        by.setdefault(w[j], []).append((i, j, j == len(w) - 1))
start = [[-1] * len(w) for w in words]
dp = [0] * (n + 1)
for i, c in enumerate(s):
    best = dp[i]
    for wi, j, last in by.get(c, ()):
        row = start[wi]
        row[j] = i if j == 0 else row[j - 1]
        if last and row[j] != -1 and i - row[j] < SPAN:
            cand = len(words[wi]) + dp[row[j]]
            if cand > best: best = cand
    dp[i + 1] = best
out = open(sys.argv[2]).read().split()
if not out: sys.exit("no output")
total = int(out[0])
if total != dp[n]: sys.exit(f"sum {total}, optimum {dp[n]}")
items = out[1:]
if len(items) % 3: sys.exit("items are not id start end triples")
spans, got = [], 0
for k in range(0, len(items), 3):
    wid, a, b = map(int, items[k:k + 3])
    if not (1 <= wid <= m and 1 <= a <= b <= n): sys.exit(f"bad item {wid} {a} {b}")
    w, seg = words[wid - 1], s[a - 1:b]
    if b - a + 1 > SPAN: sys.exit(f"item {wid} {a} {b} longer than {SPAN}")
    if seg[0] != w[0] or seg[-1] != w[-1]: sys.exit(f"item {wid} {a} {b} has the wrong ends")
    it = iter(seg)
    if not all(ch in it for ch in w): sys.exit(f"item {wid} {a} {b} does not contain the word")
    spans.append((a, b)); got += len(w)
spans.sort()
if any(spans[k][1] >= spans[k + 1][0] for k in range(len(spans) - 1)): sys.exit("items overlap")
if got != total: sys.exit(f"items add up to {got}, not {total}")
print("ok")
