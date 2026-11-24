"""noi06p5 checker: `checker.py input output answer`. The output is a simple path along the
roads, given as p and then p attractions, and it is at least as long as the answer's (the
task is scored against the best known route, so a longer one also passes)."""
import sys

d = open(sys.argv[1]).read().split()
n, m = int(d[1]), int(d[2])
edges = {(min(a, b), max(a, b)) for a, b in zip(map(int, d[3:3 + 2 * m:2]), map(int, d[4:4 + 2 * m:2]))}
out, ans = open(sys.argv[2]).read().split(), open(sys.argv[3]).read().split()
if not out or not all(x.isdigit() for x in out):
    sys.exit(1)
path = list(map(int, out[1:]))
if int(out[0]) != len(path) or not path or len(set(path)) != len(path) or not all(1 <= v <= n for v in path):
    sys.exit(1)
if any((min(a, b), max(a, b)) not in edges for a, b in zip(path, path[1:])):
    sys.exit(1)
sys.exit(0 if len(path) >= int(ans[0]) else 1)
