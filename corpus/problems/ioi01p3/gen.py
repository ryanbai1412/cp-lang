import sys, random
# statement: "W" and a valid word (A..Y in a 5x5 grid, rows and columns increasing), or "N" and
# a rank 1..=701149020 (the number of valid words).
if sys.argv[1] == "--max":
    print("N")
    print(701149020)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if r.random() < 0.5:
    print("N")
    print(r.randint(1, 701149020 if S >= 16 else 50))
    sys.exit()
rows = [0] * 5
grid = [[""] * 5 for _ in range(5)]
for ch in "ABCDEFGHIJKLMNOPQRSTUVWXY":
    ok = [i for i in range(5) if rows[i] < 5 and (i == 0 or rows[i - 1] > rows[i])]
    i = r.choice(ok)
    grid[i][rows[i]] = ch
    rows[i] += 1
print("W")
print("".join("".join(row) for row in grid))
