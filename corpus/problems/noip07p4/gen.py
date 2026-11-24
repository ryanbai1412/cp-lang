"""gen.py SEED SIZE | --max: NOIP 2007 P4. A tree on n in 5..=2e6 nodes with edge lengths in
1..=1000 and 0 <= s < 2^31. Small sizes mix random, path-like and star-like trees with s from 0
to past the diameter, with relabelled nodes; --max is n = 2e6: a 1e6-node path whose other nodes
hang in short branches, and an s that covers a fraction of the path, so the window slides."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2_000_000 if MAX else r.randint(5, min(2_000_000, 5 + 3 * S))
wh = 1000 if MAX else r.choice([3, 1000])
kind = 1 if MAX else r.randint(0, 2)
spine = n // 2 if MAX else r.randint(1, n)
edges = []
for v in range(2, n + 1):
    if kind == 0:
        p = r.randint(1, v - 1)
    elif kind == 1:
        p = v - 1 if v <= spine else r.randint(max(1, v - 3), v - 1) if r.random() < 0.5 else r.randint(1, spine)
    else:
        p = r.randint(1, min(v - 1, 3))
    edges.append((p, v, r.randint(1, wh)))
lab = list(range(1, n + 1))
r.shuffle(lab)
lab = [0] + lab
s = 250_000 * 500 if MAX else r.choice([0, r.randint(0, 3 * wh), r.randint(0, 2**31 - 1), r.randint(0, wh * S)])
out = [f"{n} {s}"]
r.shuffle(edges)
out += [f"{lab[a]} {lab[b]} {w}" if r.random() < 0.5 else f"{lab[b]} {lab[a]} {w}" for a, b, w in edges]
sys.stdout.write("\n".join(out) + "\n")
