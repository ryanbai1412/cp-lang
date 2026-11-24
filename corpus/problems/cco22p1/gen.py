"""CCO 2022 P1 inputs: `gen.py seed size` (a short line of few students, random queries), or
`gen.py --max` (N = K = 3000, every student once so every range works, Q = 1e6)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 3000 if MAX else r.randint(2, max(2, S))
k = n if MAX else r.randint(2, n)
q = 1_000_000 if MAX else r.randint(1, 3 * S + 1)
if MAX:
    line = r.sample(range(1, n + 1), n)
else:
    line = [r.randint(1, k) for _ in range(n)]
out = [f"{n} {k} {q}", " ".join(map(str, line))]
for _ in range(q):
    x = r.randint(1, n - 1)
    out.append(f"{x} {r.randint(x + 1, n)}")
print("\n".join(out))
