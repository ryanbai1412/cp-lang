"""noi07p3 checker: `checker.py input output answer`. Each move takes a battalion one cell
up, down, left or right onto an empty cell inside the map; afterwards no battalion is on a
research cell and no path from the border avoiding battalions reaches one; the first line is
the move count, which is at most the answer's (the task is scored against thresholds on it,
so a shorter plan also passes)."""
import sys
from collections import deque

d = open(sys.argv[1]).read().split()
n, m = int(d[1]), int(d[2])
g = d[3:3 + n]


def moves(path):
    s = open(path).read().split()
    if not s or not all(x.isdigit() for x in s) or len(s) != 1 + 4 * int(s[0]):
        return None
    occ = {(r, c) for r in range(n) for c in range(m) if g[r][c] == "#"}
    v = list(map(int, s[1:]))
    for i in range(0, len(v), 4):
        a, b, x, y = v[i] - 1, v[i + 1] - 1, v[i + 2] - 1, v[i + 3] - 1
        if (a, b) not in occ or (x, y) in occ or not (0 <= x < n and 0 <= y < m) or abs(a - x) + abs(b - y) != 1:
            return None
        occ.remove((a, b))
        occ.add((x, y))
    if any(g[r][c] == "O" for r, c in occ):
        return None
    seen = {(r, c) for r in range(n) for c in range(m) if (r in (0, n - 1) or c in (0, m - 1)) and (r, c) not in occ}
    q = deque(seen)
    while q:
        r, c = q.popleft()
        if g[r][c] == "O":
            return None
        for x, y in ((r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)):
            if 0 <= x < n and 0 <= y < m and (x, y) not in occ and (x, y) not in seen:
                seen.add((x, y))
                q.append((x, y))
    return int(s[0])


mine, ref = moves(sys.argv[2]), moves(sys.argv[3])
sys.exit(0 if mine is not None and ref is not None and mine <= ref else 1)
