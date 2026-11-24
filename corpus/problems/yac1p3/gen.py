#!/usr/bin/env python3
"""Hidden case for YAC 1 P3: `gen.py SEED [SIZE]` prints `N T SEED`; the interactor plays
random legal moves seeded by SEED."""
import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "100"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 100
n = min(S, 100) if big else r.randint(1, min(S, 100))
t = 100 if big else r.randint(1, min(S, 100))
print(n, t, r.randint(1, 10**9))
