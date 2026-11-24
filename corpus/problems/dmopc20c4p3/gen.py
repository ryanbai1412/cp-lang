import sys, random
# statement: n, m <= 2e5; coordinates 1..=1e9; Roomba y distinct, cord x distinct.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(200000, S)))
m = 200000 if big else r.randint(1, max(1, min(200000, S)))
C = 10 ** 9 if big else r.choice([n + m + 2, 10 ** 9])
ys = r.sample(range(1, C + 1), n); xs = r.sample(range(1, C + 1), m)
print(n, m)
for y in ys: print(r.randint(1, C), y)
for x in xs: print(x, r.randint(1, C))
