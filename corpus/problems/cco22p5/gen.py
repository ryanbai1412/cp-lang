"""CCO 2022 P5 inputs: `gen.py seed size`, or `gen.py --max` (N = A = B = 2e5, K about half of all
pairs). Small cases use few distinct levels so ties and level 0 come up."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 200000 if MAX else r.randint(1, S + 2)
ma, mb = (200000, 200000) if MAX else (r.randint(0, 2 * S), r.randint(0, 2 * S))
top = 10**9 if MAX else r.choice([3, 10**9])
k = r.randint(1, n * (n - 1) // 2) if n > 1 and r.randint(0, 4) else r.randint(1, n * n)
out = [f"{n} {ma} {mb} {k}"]
for _ in range(ma + mb): out.append(f"{r.randint(1, n)} {r.randint(1, n)} {r.randint(0, top)}")
print("\n".join(out))
