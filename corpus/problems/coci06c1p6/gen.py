"""COCI 2006/07 C1 #6 inputs: `gen.py seed size` (R, C up to `size`, cells biased so mirror
squares appear) or `gen.py --max` (a 300 x 300 grid, symmetric under a 180-degree turn except
for one cell, so most sizes come close to a killer)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 300 if MAX else max(1, min(300, int(sys.argv[2])))
R, C = (300, 300) if MAX else (r.randint(1, S), r.randint(1, S))
p = r.choice([0.05, 0.5, 0.95])
g = [[int(r.random() < p) for _ in range(C)] for _ in range(R)]
if MAX or r.random() < 0.5:
    for i in range(R):
        for j in range(C):
            g[R - 1 - i][C - 1 - j] = g[i][j]
    if MAX: g[0][0] ^= 1
print(R, C)
for row in g: print("".join(map(str, row)))
