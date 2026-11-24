import sys, random
# statement: 0 <= Q <= 150000, 0 <= M, K <= 2000, fields 3..=150, strips 2..=150, Q <= all trees.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "150000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 150000
m = 2000 if big else r.randint(0, max(0, min(S, 2000)))
k = 2000 if big else r.randint(0, max(0, min(S, 2000)))
top = 150 if big or r.random() < 0.5 else 6
fields = [r.randint(3, top) for _ in range(m)]
strips = [r.randint(2, top) for _ in range(k)]
total = sum(fields) + sum(strips)
# an odd Q no subset of even fields reaches makes the subset sum scan everything
if big:
    fields = [2 * r.randint(2, 75) for _ in range(m)]
    q = 150000 - 1
else:
    q = r.randint(0, min(total, 150000))
print(q, m, k)
print(*fields)
print(*strips)
