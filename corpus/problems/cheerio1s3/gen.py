import sys, random, math
# gen.py seed size: 2 <= n <= min(size, 5000) points, 0 <= t_1 < ... < t_n <= 1e9, 0 <= p <= 1e9.
# No arguments, or size >= 5000, gives n = 5000; the seed picks the shape: random values in a
# small box (many collinear ties), random full-range values, or a convex or concave curve
# (every pair is a trendline: about 1.25e7 pairs).
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
S = min(int(sys.argv[2]) if len(sys.argv) > 2 else 5000, 5000)
r = random.Random(seed)
n = S if S == 5000 else r.randint(2, max(2, S))
kind = seed % 4
H = 10 ** 9
if kind == 0:
    W = max(n, 3 * n if S < 5000 else 10 ** 4)
    ts = sorted(r.sample(range(0, W + 1), n)); ps = [r.randint(0, 6 if S < 5000 else 10 ** 4) for _ in ts]
elif kind == 1:
    ts = sorted(r.sample(range(0, H + 1), n)); ps = [r.randint(0, H) for _ in ts]
else:
    # t_i = i * step, p = c * (i - n/2)^2 rescaled into [0, 1e9]; kind 3 flips it.
    step = H // (n - 1) if n > 1 else 1
    ts = [i * step for i in range(n)]
    mid = (n - 1) / 2
    peak = max(1, math.ceil(mid * mid))
    ps = [int((i - mid) ** 2 * (H // peak)) for i in range(n)]
    if kind == 3: ps = [H - p for p in ps]
print(n)
for t, p in zip(ts, ps): print(t, p)
