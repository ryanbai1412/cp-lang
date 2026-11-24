import sys, random
# statement: 1 <= m <= 1e5, 1 <= t <= 50, names of 1..=1e5 letters in 1..=m.
# Names repeat a short random block so they have many borders.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
m = 100000 if big else r.choice([1, 2, 3, r.randint(1, 100000)])
t = 50 if big else r.randint(1, 5)
print(m, t)
for _ in range(t):
    n = S if big else r.randint(1, 2 * S)
    block = [r.randint(1, min(m, 3)) for _ in range(r.randint(1, 4))]
    name = [block[i % len(block)] if r.random() < 0.9 else r.randint(1, m) for i in range(n)]
    print(n); print(*name)
