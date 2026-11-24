import sys, random
# seed size: n <= size + 1 distinct trees on a w x h patch (|x| <= w, 1 <= y <= h) small enough
# that rows, columns and diagonals chain, sometimes shifted far out; --max: n = 50000 trees filling
# a 200 x 250 grid, every row 200 long and every tree on long vertical and diagonal lines
if sys.argv[1] == "--max":
    print(50000)
    for y in range(1, 251):
        for x in range(-100, 100):
            print(x, y)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(50000, S + 1))
w = r.randint(1, max(1, int(n ** 0.5)) + 1); h = r.randint(max(1, -(-n // (2 * w + 1))), n + 1)
pts = set()
while len(pts) < n:
    pts.add((r.randint(-w, w), r.randint(1, h)))
pts = list(pts); r.shuffle(pts)
dx = r.choice([0, 0, 0, 10**9 - w]) * r.choice([-1, 1])
print(n)
for x, y in pts:
    print(x + dx, y)
