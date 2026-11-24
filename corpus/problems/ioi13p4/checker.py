#!/usr/bin/env python3
# checker.py input output [answer]: the sample grader's verdict `Accepted: q` with q <= 70000;
# the number of tries depends on the search order, so it is not compared with the answer's.
import sys
out = open(sys.argv[2]).read().split()
if len(out) != 2 or out[0] != "Accepted:": sys.exit(f"verdict {' '.join(out)!r}")
if not 0 <= int(out[1]) <= 70000: sys.exit(f"{out[1]} tries, limit 70000")
print("ok")
