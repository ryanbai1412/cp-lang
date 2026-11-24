"""mccc5j5 generator: N Q, N points, then Q lines m k (m < N, k <= 1e6).
--max: N = 1e6, Q = 5e5."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
n = 10**6 if big else r.randint(1, 1 + size * 4)
q = 5 * 10**5 if big else r.randint(1, 1 + size * 3)
top = 10**9 if big else r.choice([10, 10**9])
out = [f"{n} {q}", " ".join(str(r.randint(0, top)) for _ in range(n))]
kmax = 10**6 if big else r.choice([n, 2 * n + 3, 10**6])
out += [f"{r.randrange(n)} {r.randint(0, kmax)}" for _ in range(q)]
print("\n".join(out))
