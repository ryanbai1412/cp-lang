"""gen.py SEED SIZE | --max: NOIP 1999 P3. Reals with one decimal: D1, C, D2, P in 1..=500, N in
1..=6, then N stations in increasing distance D_i in 1..D1 with prices P_i in 1..=500 (the order
the original data uses). Small sizes pick tanks near the station gaps so some cases have No
Solution; --max is N = 6 with a tank that reaches every later station."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
f = lambda lo, hi: r.randint(lo * 10, hi * 10) / 10
d1 = 500.0 if MAX else f(2, min(500, 10 + 50 * S))
n = 6 if MAX else r.randint(1, 6)
ds = sorted(r.sample(range(10, int(d1 * 10)), n))
d2 = 50.0 if MAX else f(1, 50)
c = 10.0 if MAX else f(1, 20)
print(f"{d1:.1f} {c:.1f} {d2:.1f} {f(1, 500):.1f} {n}")
for d in ds:
    print(f"{d / 10:.1f} {f(1, 500):.1f}")
