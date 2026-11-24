#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: SDOI 2017 Correlation Analysis. n, m <= 1e5, x and y in
-1e5..=1e5, then m ops `1 l r`, `2 l r S T` or `3 l r S T` with |S|, |T| <= 1e5. Every query's
x values are not all equal (the statement promises a nonzero denominator): small cases are
simulated and a degenerate query becomes an add; --max (n = m = 1e5) asks ranges of length
>= 3 over distinct random values."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 10**5)
n = 10**5 if MAX else r.randint(2, max(2, size))
m = 10**5 if MAX else r.randint(1, size)
B = 10**5
x = r.sample(range(-B, B + 1), n) if MAX else [r.randint(-3, 3) if size < 20 else r.randint(-B, B) for _ in range(n)]
y = [r.randint(-B, B) for _ in range(n)]
print(n, m)
print(*x)
print(*y)
ops = []
for _ in range(m):
    l = r.randint(1, n - 2 if MAX else n)
    hi = n if not MAX else min(n, l + r.choice([2, 50, n]))
    rr = r.randint(l + 2, hi) if MAX else r.randint(l, n)
    kind = r.randint(1, 3)
    s, t = r.randint(-B, B), r.randint(-B, B)
    if kind == 1 and not MAX and len(set(x[l - 1:rr])) < 2:
        kind = 2
    if kind == 1:
        ops.append(f"1 {l} {rr}")
        continue
    ops.append(f"{kind} {l} {rr} {s} {t}")
    if not MAX:
        for i in range(l - 1, rr):
            x[i], y[i] = (x[i] + s, y[i] + t) if kind == 2 else (s + i + 1, t + i + 1)
print("\n".join(ops))
