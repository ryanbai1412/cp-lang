import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
# `seed size`: n <= size + 1 points with large random coordinates (no near ties); size >= 1e5: 1e5.
seed, S = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed); W = 10 ** 9
n = 10 ** 5 if S >= 10 ** 5 else r.randint(1, S + 1)
print(n, r.randint(0, 359))
for _ in range(n):
    x, y = 0, 0
    while x == 0 and y == 0: x, y = r.randint(-W, W), r.randint(-W, W)
    print(x, y)
