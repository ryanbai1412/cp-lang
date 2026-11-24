#!/usr/bin/env python3
# checker.py input output [answer]: -1 exactly when the reference says -1; otherwise a length L and L colours
# that repeat K times in a row somewhere in the lamps.
import sys
t = open(sys.argv[1]).read().split(); n, k = int(t[0]), int(t[1]); a = t[2:2 + n]
out = open(sys.argv[2]).read().split(); ans = open(sys.argv[3]).read().split() if len(sys.argv) > 3 else None
if out == ["-1"]:
    sys.exit(0 if ans is None or ans == ["-1"] else 1)
L = int(out[0]); p = out[1:]
ok = len(p) == L >= 1 and any(a[s:s + k * L] == p * k for s in range(n - k * L + 1)) and (ans is None or ans != ["-1"])
sys.exit(0 if ok else 1)
