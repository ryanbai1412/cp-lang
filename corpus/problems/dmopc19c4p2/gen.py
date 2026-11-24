import sys, random
# statement: 2 <= n <= 15; the marble starts on one of the two distinct empty cells.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "15"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = 15 if S >= 15 else r.randint(2, max(2, min(15, S)))
cells = [(x, y) for x in range(1, n + 1) for y in range(1, n + 1)]
e1, e2 = r.sample(cells, 2)
if S >= 15: e1, e2 = (1, 1), (1, 2)
t = (n, n) if S >= 15 else r.choice(cells)
s = r.choice([e1, e2])
print(n); print(*s); print(*t); print(*e1); print(*e2)
