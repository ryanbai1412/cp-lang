#!/usr/bin/env python3
# checker.py input output [answer]: at most 1350 queries, then k wires whose connectivity equals the
# hidden design's (any such design is accepted).
import sys
t = list(map(int, open(sys.argv[1]).read().split())); n, m = t[0], t[1]
def parts(pairs):
    up = list(range(n + 1))
    def find(x):
        while up[x] != x:
            up[x] = up[up[x]]; x = up[x]
        return x
    for a, b in pairs:
        up[find(a)] = find(b)
    return [find(x) for x in range(1, n + 1)]
def canon(roots):
    seen = {}
    return [seen.setdefault(x, len(seen)) for x in roots]
try:
    out = list(map(int, open(sys.argv[2]).read().split()))
    used, k = out[0], out[1]; w = out[2:]
except (IndexError, ValueError):
    sys.exit(1)
if used > 1350 or len(w) != 2 * k or any(not 1 <= x <= n for x in w):
    sys.exit(1)
hidden = parts(zip(t[2:2 + 2 * m:2], t[3:3 + 2 * m:2]))
built = parts(zip(w[::2], w[1::2]))
sys.exit(0 if canon(hidden) == canon(built) else 1)
