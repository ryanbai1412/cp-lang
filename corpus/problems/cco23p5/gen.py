"""CCO 2023 P5 inputs: `gen.py seed size`, or `gen.py --max` (N = 2e5, K = 2, a random tree with
long paths). Small cases mix K = 1..3 and small profits, so ties come up."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 200000 if MAX else r.randint(1, S + 1)
k = 2 if MAX else r.randint(1, 3)
top = 10**9 if MAX or r.randint(0, 1) else 5
out = [f"{n} {k}"]
for v in range(2, n + 1): out.append(f"{r.randint(max(1, v - 5), v - 1) if MAX else r.randint(1, v - 1)} {v}")
out.append(" ".join(str(r.randint(1, top)) for _ in range(n)))
print("\n".join(out))
