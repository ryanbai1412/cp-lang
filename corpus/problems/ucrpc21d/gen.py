import sys, random
from collections import deque
# seed size: an r x c board split into exactly four connected regions (rc >= 4)
if sys.argv[1:] == ["--max"]:
    # the full 1000 x 1000 board cut into four winding regions
    sys.argv[1:] = ['0', '1000']
rng = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
while True:  # size >= 1000: the full 1000 x 1000 board
    r, c = (S, S) if S >= 1000 else (rng.randint(1, S), rng.randint(1, S))
    if r * c >= 4: break
region = [[-1] * c for _ in range(r)]
cells = rng.sample([(i, j) for i in range(r) for j in range(c)], 4)
fronts = []
for k, (i, j) in enumerate(cells):
    region[i][j] = k; fronts.append([(i, j)])
left = r * c - 4
while left:  # grow a random region from a random frontier cell
    k = rng.randrange(4)
    f = fronts[k]
    if not f: continue
    t = rng.randrange(len(f)); i, j = f[t]
    nb = [(i + di, j + dj) for di, dj in ((1, 0), (-1, 0), (0, 1), (0, -1))
          if 0 <= i + di < r and 0 <= j + dj < c and region[i + di][j + dj] < 0]
    if not nb: f[t] = f[-1]; f.pop(); continue
    a, b = rng.choice(nb); region[a][b] = k; f.append((a, b)); left -= 1
g = [['.'] * (2 * c + 1) for _ in range(2 * r + 1)]
def reg(i, j): return region[i][j] if 0 <= i < r and 0 <= j < c else -1
for y in range(2 * r + 1):
    for x in range(2 * c + 1):
        if y % 2 == 0 and x % 2 == 1:
            i, j = y // 2, x // 2
            if reg(i - 1, j) != reg(i, j): g[y][x] = '-'
        elif y % 2 == 1 and x % 2 == 0:
            i, j = y // 2, x // 2
            if reg(i, j - 1) != reg(i, j): g[y][x] = '|'
for y in range(0, 2 * r + 1, 2):
    for x in range(0, 2 * c + 1, 2):
        if any(0 <= y + dy <= 2 * r and 0 <= x + dx <= 2 * c and g[y + dy][x + dx] in '-|'
               for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1))):
            g[y][x] = '+'
out = [f"{r} {c}"] + [''.join(row) for row in g]
sys.stdout.write('\n'.join(out) + '\n')
