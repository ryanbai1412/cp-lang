import sys, random
# statement: 2 <= n <= 20000 cars of types 1..m (2 <= m <= 50, each type present), 2 <= w <= m.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "20000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 20000
m = 50 if big else r.randint(2, max(2, min(S, 50)))
n = 20000 if big else r.randint(m, max(m, min(S * 4, 20000)))
w = 2 if big else r.randint(2, m)
cars = list(range(1, m + 1)) + [r.randint(1, m) for _ in range(n - m)]
r.shuffle(cars)
print(n, m, w)
print(*cars)
