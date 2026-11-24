"""gen.py SEED SIZE | --max: AtCoder DP Q (Flowers). n flowers (1 <= n <= 2e5), heights a permutation
of 1..n, beauties 1..1e9. --max is n = 2e5, heights ascending with beauty 1e9 (the 2e14 answer) after a
shuffled half, so the tree sees every height."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, 1 + S)
h = list(range(1, n + 1))
if MAX:
    half = h[: n // 2]
    r.shuffle(half)
    h = half + h[n // 2:]
else:
    r.shuffle(h)
hi = 10**9 if MAX or r.random() < 0.5 else 10
print(n)
print(*h)
print(*(10**9 if MAX else r.randint(1, hi) for _ in range(n)))
