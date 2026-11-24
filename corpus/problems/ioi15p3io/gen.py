"""ioi15p3io (Teams, stdin version): `gen.py SEED SIZE`, or `gen.py --max`.

N, then N lines "A B" (1 <= A <= B <= N <= 5e5), then Q, then Q lines "M K[0] .. K[M-1]"
(1 <= M <= N, 1 <= K <= N, Q <= 2e5, sum of M <= 2e5). The output is 1 or 0 per day.
--max: N = 5e5, Q = 2e5 single-project days (sum of M at its bound), short intervals so the
answers mix.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, q, s = 500_000, 200_000, 200_000
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(500_000, size))
    q = rng.randint(1, min(200_000, size))
    s = rng.randint(q, min(200_000, max(q, 2 * size)))
spread = rng.choice([1, 3, n]) if not MAX else 40
out = [str(n)]
for _ in range(n):
    a = rng.randint(1, n)
    b = min(n, a + rng.randint(0, spread))
    out.append(f"{a} {b}")
out.append(str(q))
ms = [1] * q
for _ in range(s - q):
    ms[rng.randrange(q)] += 1
cap = 25 if MAX else rng.choice([2, spread + 2, n])
for m in ms:
    m = min(m, n)
    out.append(" ".join(map(str, [m, *(rng.randint(1, min(n, cap)) for _ in range(m))])))
print("\n".join(out))
