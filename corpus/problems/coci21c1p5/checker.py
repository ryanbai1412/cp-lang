#!/usr/bin/env python3
# checker.py input output answer: the answer's count and length, then that many index-disjoint
# increasing subsequences of that length.
import sys
tok = open(sys.argv[1]).read().split(); n = int(tok[0]); p = [0] + list(map(int, tok[1:1 + n]))
out = list(map(int, open(sys.argv[2]).read().split())); ans = list(map(int, open(sys.argv[3]).read().split()))
if out[:2] != ans[:2]: sys.exit(f"count/length {out[:2]}, expected {ans[:2]}")
c, l = ans[0], ans[1]; rest = out[2:]
if len(rest) != c * l: sys.exit("wrong number of indices")
used = set()
for k in range(c):
    ch = rest[k * l:(k + 1) * l]
    if any(not 1 <= i <= n for i in ch) or used & set(ch) or len(set(ch)) != l: sys.exit("index out of range or reused")
    used |= set(ch)
    if any(not (a < b and p[a] < p[b]) for a, b in zip(ch, ch[1:])): sys.exit("a chain is not increasing")
print("ok")
