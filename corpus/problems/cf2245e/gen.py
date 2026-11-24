import sys, random
if sys.argv[1] == "--max":  # sum n = 2e5: one n = 2e5 path on shuffled labels (deepest tree)
    rng = random.Random(0); n = 200000; lab = list(range(1, n + 1)); rng.shuffle(lab)
    print(1); print(n); print("\n".join(f"{lab[i]} {lab[i + 1]}" for i in range(n - 1))); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 6; print(t)
for _ in range(t):
    n = r.randint(2, 4 * S + 1); print(n); m = r.choice([1, 2, 3, n])
    for v in range(2, n + 1): print(r.randint(max(1, v - m), v - 1), v)
