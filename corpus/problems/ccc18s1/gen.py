"""gen.py SEED SIZE | --max: CCC 2018 S1 Voronoi Villages. N (3 <= N <= 100) distinct village
positions in [-1e9, 1e9], in random order. Small sizes use a narrow range so ties and half-units
show up; --max is N = 100 spread over the full range, both ends included."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100 if MAX else r.randint(3, min(100, 3 + S))
hi = 10**9 if MAX or r.random() < 0.4 else max(n, 2 * S)
v = r.sample(range(-hi, hi + 1), n)
if MAX:
    v[:2] = [-hi, hi]
    r.shuffle(v)
print(n)
print("\n".join(map(str, v)))
