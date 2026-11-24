"""CMT 2021 (Peter) P2 inputs: `gen.py seed size` (n up to `size` points, clustered, spread or
collinear) or `gen.py --max` (2e5 points spread over a 2e6 square)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 200000 if MAX else r.randint(2, max(2, int(sys.argv[2])))
w = 10**6 if MAX else r.choice([3, 100, 10**6])
shape = "spread" if MAX else r.choice(["spread", "line", "cluster"])
print(n)
for _ in range(n):
    x, y = r.randint(-w, w), r.randint(-w, w)
    if shape == "line": y = x
    if shape == "cluster": x, y = x // 50, y // 50
    print(x, y)
