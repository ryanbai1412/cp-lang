"""noi04p3 checker: `checker.py input output answer`. The verdict matches the answer's; a
possible plan costs the answer's minimum, that cost is what its flips add up to, and every
request has a monotone path, so its shortest path is the Manhattan distance."""
import sys

d = open(sys.argv[1]).read().split()
m, n, h0, v0 = int(d[0]), int(d[1]), d[2], d[3]
ch, cv = list(map(int, d[4:4 + m])), list(map(int, d[4 + m:4 + m + n]))
k = int(d[4 + m + n])
reqs = [tuple(map(int, d[5 + m + n + 4 * i:9 + m + n + 4 * i])) for i in range(k)]
out, ans = open(sys.argv[2]).read().split(), open(sys.argv[3]).read().split()
if not out or out[0] != ans[0]:
    sys.exit(1)
if ans[0] == "impossible":
    sys.exit(0 if len(out) == 1 else 1)
if len(out) != 4 or not out[1].lstrip("-").isdigit() or int(out[1]) != int(ans[1]):
    sys.exit(1)
h, v = out[2], out[3]
if len(h) != m or len(v) != n or set(h) - set("EW") or set(v) - set("NS"):
    sys.exit(1)
if sum(c for a, b, c in zip(h, h0, ch) if a != b) + sum(c for a, b, c in zip(v, v0, cv) if a != b) != int(out[1]):
    sys.exit(1)


def monotone(x1, y1, x2, y2):
    dx, dy = (x2 > x1) - (x2 < x1), (y2 > y1) - (y2 < y1)
    east, south = dy > 0, dx > 0
    seen = {(x1, y1)}
    todo = [(x1, y1)]
    while todo:
        x, y = todo.pop()
        if (x, y) == (x2, y2):
            return True
        nxt = []
        if y != y2 and (h[x - 1] == "E") == east:
            nxt.append((x, y + dy))
        if x != x2 and (v[y - 1] == "S") == south:
            nxt.append((x + dx, y))
        for p in nxt:
            if p not in seen:
                seen.add(p)
                todo.append(p)
    return False


sys.exit(0 if all(monotone(*q) for q in reqs) else 1)
