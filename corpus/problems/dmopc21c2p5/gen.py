import sys, random
# statement: a tree on n <= 3000 nodes, 0 <= k < n.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "3000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 3000
n = 3000 if big else r.randint(1, max(1, min(3000, S)))
k = n // 2 if big else r.randrange(n)
shape = r.choice(["random", "star", "path", "caterpillar"])
print(n, k)
for v in range(2, n + 1):
    if shape == "star": p = 1
    elif shape == "path": p = v - 1
    elif shape == "caterpillar": p = max(1, v - 1 - r.randint(0, 1) * (v % 3))
    else: p = r.randint(1, v - 1)
    print(p, v) if r.random() < 0.5 else print(v, p)
