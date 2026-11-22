import sys
inp, out, ans = [open(f).read().split() for f in sys.argv[1:4]]
n, m = int(inp[0]), int(inp[1]); g = inp[2:2 + n]
if ans[0] == "NO": sys.exit(0 if out == ["NO"] else 1)
if out[:2] != ans[:2]: sys.exit(1)
k = int(ans[1]); path = out[2] if len(out) > 2 else ""
if len(out) != 3 or len(path) != k: sys.exit(1)
r, c = [(i, j) for i in range(n) for j in range(m) if g[i][j] == 'A'][0]
for ch in path:
    dr, dc = {'U': (-1, 0), 'D': (1, 0), 'L': (0, -1), 'R': (0, 1)}[ch]; r += dr; c += dc
    if not (0 <= r < n and 0 <= c < m) or g[r][c] == '#': sys.exit(1)
sys.exit(0 if g[r][c] == 'B' else 1)
