"""CEOI 2019 practice Separator inputs: `gen.py seed size` (up to `size` appends of distinct
values, rising runs and drops mixed) or `gen.py --max` (q = 1e6, the same mix). Each b_i is
chosen so the decoded a_i = (b_i + s_{i-1}) mod 1e9 is the planned value."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
q = 10**6 if MAX else r.randint(1, max(1, int(sys.argv[2])))
M = 10**9
vals = r.sample(range(M), q) if q > 1000 else r.sample(range(1, 4 * q + 1), q)
if r.random() < .5:
    vals.sort()
    for _ in range(q // 10 + 1):
        i, j = r.randrange(q), r.randrange(q)
        vals[i], vals[j] = vals[j], vals[i]
kept, high, out = [], -1, []
for x in vals:
    out.append((x - len(kept)) % M)
    if x > high:
        high = x; kept.append(x)
    else:
        while kept and kept[-1] > x: kept.pop()
print(q)
print("\n".join(map(str, out)))
