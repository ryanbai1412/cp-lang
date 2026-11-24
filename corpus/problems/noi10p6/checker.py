"""noi10p6 checker: `checker.py input output answer`. The plan lists k distinct shrimp with
times in [0, T] that never go backwards; Nemo can swim each leg at speed V, meets each
shrimp where it is at that time, outweighs it then, and line 2 is the eaten weight; that
weight is at least the answer's (the task is scored against thresholds on it, so a heavier
plan also passes). Reals match to 1e-4, as the statement grades them."""
import math, sys

EPS = 1e-4
d = open(sys.argv[1]).read().split()
w0, V, T, x0, y0 = map(float, d[1:6])
n = int(d[6])
S = [tuple(map(float, d[7 + 5 * i:12 + 5 * i])) for i in range(n)]


def eaten(path):
    s = open(path).read().split()
    try:
        k, total = int(s[0]), float(s[1])
        rows = [(float(s[2 + 4 * i]), float(s[3 + 4 * i]), float(s[4 + 4 * i]), int(s[5 + 4 * i])) for i in range(k)]
    except (ValueError, IndexError):
        return None
    if len(s) != 2 + 4 * k or len({r[3] for r in rows}) != k:
        return None
    t0, px, py, w = 0.0, x0, y0, w0
    for t, x, y, i in rows:
        if not 1 <= i <= n or t < t0 - EPS or t > T + EPS:
            return None
        sw, sx, sy, p, q = S[i - 1]
        if abs(sx + p * t - x) > EPS or abs(sy + q * t - y) > EPS:
            return None
        if math.hypot(x - px, y - py) > V * max(0.0, t - t0) + EPS or sw >= w:
            return None
        t0, px, py, w = max(t0, t), x, y, w + sw
    return w - w0 if abs(w - w0 - total) <= EPS * max(1.0, abs(total)) else None


mine, ref = eaten(sys.argv[2]), eaten(sys.argv[3])
sys.exit(0 if mine is not None and ref is not None and mine >= ref - EPS * max(1.0, abs(ref)) else 1)
