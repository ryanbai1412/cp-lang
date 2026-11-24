import sys, random
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
seed = int(sys.argv[1])
if size <= 20:
    n = rng.randint(2, size + 2)
    hi = rng.choice([2, 5, 100])
    pts = [(rng.randint(-hi, hi), rng.randint(-hi, hi), rng.randint(0, 3)) for _ in range(n)]
else:
    n = size
    mode = seed % 3
    if mode == 0:  # extremes only: largest distances
        pts = [(rng.choice([-10**9, 10**9]), rng.choice([-10**9, 10**9]), rng.randint(0, 3)) for _ in range(n)]
    elif mode == 1:
        pts = [(rng.randint(-10**9, 10**9), rng.randint(-10**9, 10**9), rng.randint(0, 3)) for _ in range(n)]
    else:
        pts = [(rng.randint(-10**9, 10**9), rng.randint(-10**9, 10**9), rng.choice([0, 2])) for _ in range(n)]
out = [str(n)] + [f"{x} {y} {v}" for x, y, v in pts]
print("\n".join(out))
