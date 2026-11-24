import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "50"]
# seed size: n = size points (or fewer), k in 1..=4, coordinates 0..=500; some cases place
# four clusters as a pinwheel, the arrangement no straight cut separates
rng = random.Random(int(sys.argv[1])); S = min(int(sys.argv[2]), 50)
n = S if S >= 50 or rng.random() < 0.4 else rng.randint(1, S)
k = rng.choice([4, 4, rng.randint(1, 4)])
hi = rng.choice([5, 20, 500]) if S < 50 else 500
pts = []
if rng.random() < 0.5:
    arms = [((0, 0.6), (0.6, 1.0)), ((0.6, 1.0), (0.4, 1.0)), ((0.4, 1.0), (0.0, 0.4)), ((0.0, 0.4), (0.0, 0.6))]
    for i in range(n):
        (a, b), (c, d) = arms[i % 4]
        pts.append((rng.randint(int(a * hi), int(b * hi)), rng.randint(int(c * hi), int(d * hi))))
else:
    pts = [(rng.randint(0, hi), rng.randint(0, hi)) for _ in range(n)]
print(n, k)
for x, y in pts: print(x, y)
