import sys, random
if sys.argv[1] == "--max":
    # ten 100 x 100 grids (the test count is unbounded; ten is the official data's scale), few
    # walls and coins everywhere so every column scan runs its full length
    r = random.Random(1)
    for _ in range(10):
        print(100, 100)
        for i in range(100):
            print("".join("*" if r.random() < 0.02 and i < 99 else r.choice("123456789.") for _ in range(100)))
    print(0, 0)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2]) if len(sys.argv) > 2 else 100
for _ in range(r.randint(1, 4)):
    m, n = r.randint(2, max(2, S)), r.randint(2, max(2, S))
    wall = r.choice([0.0, 0.1, 0.3, 0.6])
    g = [[r.choice('*') if r.random() < wall else r.choice('.123456789') for _ in range(n)] for _ in range(m)]
    row = m - 1  # carve a right/up/down path from the bottom-left to the bottom-right
    for c in range(n):
        to = m - 1 if c == n - 1 else r.randrange(m)
        for k in range(min(row, to), max(row, to) + 1):
            if g[k][c] == '*': g[k][c] = '.'
        row = to
    print(m, n)
    for line in g: print(''.join(line))
print(0, 0)
