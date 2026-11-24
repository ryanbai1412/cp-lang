"""abc478e checker: `checker.py input expected output`. No exactly when no valid A exists;
otherwise A has values in 1..=n and meets every condition."""
import sys

inp = open(sys.argv[1]).read().split()
exp = open(sys.argv[2]).read().split()
out = open(sys.argv[3]).read().split()
n, q = int(inp[0]), int(inp[1])
conds = [tuple(map(int, inp[2 + 3 * i: 5 + 3 * i])) for i in range(q)]
if exp[0] == "No":
    sys.exit(0 if out == ["No"] else 1)
if not out or out[0] != "Yes" or len(out) != n + 1:
    sys.exit(1)
a = [0] + [int(x) for x in out[1:]]
if not all(1 <= x <= n for x in a[1:]):
    sys.exit(1)
for t, u, v in conds:
    if not (a[u] < a[v] if t else a[u] <= a[v]):
        sys.exit(1)
