"""gen.py SEED SIZE | --max: CCO 2020 P6 (Shopping Plans). N, M, K <= 2e5 items of type a_i in
[1, M] and cost c_i in [1, 1e9], each type j bounded to [x_j, y_j] with y_j <= N. Small cases
use tiny costs (many ties) and bounds that are sometimes infeasible. --max is N = M = K = 2e5
with mixed bounds."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    n = m = k = 200_000
else:
    n = r.randint(1, 1 + 2 * S)
    m = r.randint(1, min(200_000, 1 + S))
    k = r.randint(1, 1 + 3 * S)
cmax = 10**9 if MAX or r.random() < 0.3 else r.randint(1, 5)
mode = "mix" if MAX else r.choice(["one", "zero", "mix"])
a = [r.randint(1, m) for _ in range(n)]
cnt = [0] * (m + 1)
for t in a:
    cnt[t] += 1
lines = [f"{n} {m} {k}"]
lines += [f"{t} {r.randint(1, cmax)}" for t in a]
for j in range(1, m + 1):
    c = cnt[j]
    if mode == "one":
        x = y = 1
    elif mode == "zero":
        x, y = 0, r.randint(0, min(n, c + 1))
    else:
        x = r.randint(0, min(c, 2))
        y = min(n, x + r.randint(0, 3)) if not MAX or c else x
        if not MAX and r.random() < 0.05:
            x = y = min(n, c + 1)
    lines.append(f"{x} {y}")
sys.stdout.write("\n".join(lines) + "\n")
