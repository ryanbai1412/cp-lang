"""gen.py SEED SIZE | --max: AtCoder DP D (Knapsack 1). n items (1 <= n <= 100), capacity 1 <= W <= 1e5,
weights 1..W, values 1..1e9 (the answer needs 64 bits). --max is n = 100, W = 1e5, the n * W table."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100 if MAX else r.randint(1, min(100, 1 + S))
w = 100_000 if MAX else r.randint(1, 4 + 4 * S)
print(n, w)
for _ in range(n):
    print(r.randint(1, w if MAX else max(1, min(w, r.choice([w, 6])))), r.randint(1, 10**9 if MAX or r.random() < 0.5 else 20))
