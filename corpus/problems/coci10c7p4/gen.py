"""COCI 2010/11 C7 #4 inputs: `gen.py seed size` (N up to `size`, heights from a few values or
wide) or `gen.py --max` (N = 50, every cell but P a house, distinct heights up to 1e6)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 50 if MAX else r.randint(2, max(2, min(50, int(sys.argv[2]))))
cells = [(i, j) for i in range(n) for j in range(n)]
g = [["K" if MAX or r.random() < r.choice([0.1, 0.5]) else "." for _ in range(n)] for _ in range(n)]
pr, pc = r.choice(cells); g[pr][pc] = "P"
if not any("K" in row for row in g):
    i, j = r.choice([c for c in cells if c != (pr, pc)]); g[i][j] = "K"
hi = 10**6 if MAX else r.choice([3, 10, 10**6])
hs = r.sample(range(1, 10**6 + 1), n * n) if MAX else [r.randint(1, hi) for _ in range(n * n)]
print(n)
for row in g: print("".join(row))
for i in range(n): print(*hs[i * n:(i + 1) * n])
