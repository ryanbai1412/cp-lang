#!/usr/bin/env python3
# checker.py input output answer: per test, "Too hard to arrange" exactly when the answer says
# so; else the total equals the answer's, the lines join the sentences in order by single
# spaces, and their costs sum to the total; then 20 dashes.
import sys
inp = open(sys.argv[1]).read().split("\n")
out = open(sys.argv[2]).read().split("\n")
ans = open(sys.argv[3]).read().split("\n")
dashes = "-" * 20
tests = int(inp[0]); row = 1; o = 0; a = 0
for t in range(tests):
    n, l, p = map(int, inp[row].split()); row += 1
    words = [inp[row + i].strip() for i in range(n)]; row += n
    want = ans[a].strip(); got = out[o].strip()
    while ans[a].strip() != dashes: a += 1
    a += 1
    if want == "Too hard to arrange":
        if got != want: sys.exit(f"test {t + 1}: {got!r}, expected {want!r}")
        o += 1
    else:
        if got != want: sys.exit(f"test {t + 1}: total {got}, optimum {want}")
        o += 1; lines = []
        while o < len(out) and out[o].strip() != dashes:
            lines.append(out[o].rstrip("\n")); o += 1
        if sum(len(x.split(" ")) for x in lines) != n or [w for x in lines for w in x.split(" ")] != words:
            sys.exit(f"test {t + 1}: the lines do not join the sentences in order")
        cost = sum(abs(len(x) - l) ** p for x in lines)
        if cost != int(want): sys.exit(f"test {t + 1}: lines cost {cost}, printed {want}")
    if o >= len(out) or out[o].strip() != dashes: sys.exit(f"test {t + 1}: no dashes line")
    o += 1
