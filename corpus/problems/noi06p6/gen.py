"""gen.py SEED SIZE | --max: NOI 2006 The Magical Bag. 1 <= t, n <= 1000, 1 <= a_k, d <= 10,
then n draws 1 <= x_1 < ... < x_n <= 1e4 with colours 1 <= y_k <= t. Small sizes keep t
small so colours repeat. --max is t = n = 1000 with x up to 1e4 and draws that pile onto a
few colours, so neither product cancels much."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
t = 1000 if MAX else r.randint(1, min(1000, 1 + S))
n = 1000 if MAX else r.randint(1, min(1000, 1 + 2 * S))
d = r.randint(1, 10)
a = [r.randint(1, 10) for _ in range(t)]
xs = sorted(r.sample(range(1, 10_001 if MAX else min(10_000, n + 3 * S) + 1), n))
ys = [r.randint(1, min(t, 3)) if MAX and r.random() < 0.9 else r.randint(1, t) for _ in range(n)]
print(t, n, d)
print(*a)
sys.stdout.write("".join(f"{x} {y}\n" for x, y in zip(xs, ys)))
