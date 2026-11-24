"""DMOPC '21 Contest 7 P4 (Rocket Race): N, then N rockets `a b`, then Q events: `0 r` asks the
fewest rockets that reach r, `t x y` replaces rocket t (1-based) by (x, y). N, Q <= 2e5, values
in 1..=1e9.

`gen.py SEED SIZE`: N and Q up to SIZE, values small or large. `gen.py --max`: N = Q = 2e5,
nets a - b of both signs, half the events replacements (each moves a rocket's slot), half
queries with r around the reachable peaks so every binary search runs its full depth.
"""
import random
import sys

big = sys.argv[1:] == ["--max"]
g = random.Random(1 if big else int(sys.argv[1]))
if big:
    n = q = 200_000
    hi = 10**9
else:
    size = max(1, int(sys.argv[2]))
    n, q = g.randint(1, size), g.randint(1, size)
    hi = g.choice([10, 10**9])


def rocket():
    a = g.randint(1, hi)
    return a, g.randint(1, hi) if g.random() < 0.5 else g.randint(1, a)


out = [str(n)] + ["%d %d" % rocket() for _ in range(n)] + [str(q)]
for _ in range(q):
    if g.random() < 0.5:
        out.append("%d %d %d" % (g.randint(1, n), *rocket()))
    else:
        r = g.randint(1, hi) if g.random() < 0.3 else g.randint(1, min(10**9, hi * max(1, n // 4)))
        out.append(f"0 {r}")
print("\n".join(out))
