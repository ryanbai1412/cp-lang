import sys, random
# statement: 0 <= K <= 1e6 real, 1 <= N <= 1e5, positions 0 <= D <= 1e9 real, sorted ascending.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
n = 100000 if big else r.randint(1, max(1, min(S, 100000)))
k = 1000.0 if big else round(r.uniform(0, r.choice([5, 100, 10 ** 6])), 3)
span = 10 ** 9 if big or r.random() < 0.3 else r.choice([10, 1000])
d = sorted(round(r.uniform(0, span), 3) for _ in range(n))
print(f"{k:.3f}")
print(n)
print("\n".join(f"{x:.3f}" for x in d))
