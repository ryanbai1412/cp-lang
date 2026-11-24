"""gen.py SEED SIZE | --max: CCC 2020 S2 (hard data). M, N <= 4000, then the (M + 1) x (N + 1)
array with row and column 0 all zeros and values 1..=MN elsewhere. --max is 4000 x 4000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 4000 if MAX else r.randint(1, S + 1)
m = 4000 if MAX else r.randint(1, S + 1)
print(n); print(m)
top = n * m
lo = 1 if MAX or r.random() < .7 else max(1, top // 2)
out = [" ".join(["0"] * (m + 1))]
for _ in range(n): out.append("0 " + " ".join(str(r.randint(lo, top)) for _ in range(m)))
sys.stdout.write("\n".join(out) + "\n")
