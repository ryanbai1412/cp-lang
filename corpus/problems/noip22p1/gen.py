"""gen.py SEED SIZE | --max: NOIP 2022 P1. T <= 5 gardens of n x m <= 1000 x 1000 over 0/1 with
c, f in {0, 1}. Small sizes vary the mud density; --max is T = 5 mud-free 1000 x 1000 gardens with
c = f = 1 (every row run and column run is maximal)."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
t = 5 if MAX else r.randint(1, 5)
out = [f"{t} 0"]
for _ in range(t):
    n = 1000 if MAX else r.randint(1, min(1000, 1 + 2 * S))
    m = 1000 if MAX else r.randint(1, min(1000, 1 + 2 * S))
    c, f = (1, 1) if MAX else (r.randint(0, 1), r.randint(0, 1))
    p = 0.0 if MAX else r.choice([0.0, 0.1, 0.3, 0.6])
    out.append(f"{n} {m} {c} {f}")
    out += ["".join("1" if r.random() < p else "0" for _ in range(m)) for _ in range(n)]
sys.stdout.write("\n".join(out) + "\n")
