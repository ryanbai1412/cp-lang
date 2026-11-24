"""gen.py SEED SIZE | --max: NOI 2010 Nemo. "0" (the test number, as in the sample),
w0 V T x0 y0, n, then n shrimp w x y p q. The statement gives no bounds; the official files
reach n = 10000, T = 10000, V = 1000, |x|, |y| near 1e7, |p|, |q| <= 1000 and weights near
1e14, so a seed draws a scattered, still or chasing shoal of up to 10 * SIZE shrimp with
weights growing from below Nemo's. --max is 10000 moving shrimp, all lighter than Nemo, as in the official nemo10.in."""
import random, sys

if sys.argv[1] == "--max":
    r = random.Random(2010)
    n, mode, T, V = 10000, 2, 10000, 1000
    w0, span, vel = 10000, 1000, 50
else:
    r = random.Random(int(sys.argv[1]))
    n = r.randint(1, min(10000, 10 * int(sys.argv[2])))
    mode, T, V = r.randrange(3), r.choice([10, 100, 1000, 10000]), r.choice([0.05, 1, 10, 1000])
    w0, vel = r.randint(1, 1000), 1000
    span = r.choice([10, 1000, 10 ** 5]) if mode < 2 else 10 ** 4
print(0)
print(w0, V, T, 0, 0)
print(n)
w = w0
for _ in range(n):
    if sys.argv[1] == "--max":
        sw = r.randint(1, w0 - 1)
    else:
        sw = r.randint(1, max(1, int(w * r.choice([0.5, 1, 2])))) if w < 10 ** 13 else r.randint(1, 10 ** 13)
    w += sw if sw < w else 0
    x, y = r.randint(-span, span), r.randint(-span, span)
    p, q = (0, 0) if mode == 0 else (r.randint(-vel, vel), r.randint(-vel, vel)) if mode == 2 else (r.randint(-5, 5), r.randint(-5, 5))
    print(sw, x, y, p, q)
