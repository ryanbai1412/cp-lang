"""gen.py SEED SIZE | --max: DMOPC '21 Contest 6 P5 (Contraband Game). An N x M grid of N/S
with 1 <= NM <= 300 (NM at most SIZE), either shape, coloured by noise, stripes or big
blocks so long one-team runs appear. --max is 17 x 17 (the widest profile) with large blocks."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
cap = max(1, min(300, int(sys.argv[2]))) if not MAX else 300
if MAX:
    n = m = 17
else:
    area = r.randint(1, cap)
    n = r.randint(1, area)
    m = max(1, area // n)
mode = "blocks" if MAX else r.choice(["noise", "stripes", "blocks", "mono"])
p = r.random()
bh, bw = r.randint(1, 6), r.randint(1, 6)
cols = {}
rows = []
for i in range(n):
    s = []
    for j in range(m):
        if mode == "noise":
            s.append("N" if r.random() < p else "S")
        elif mode == "stripes":
            s.append("NS"[(i + j * (bh % 2)) % 2] if r.random() > 0.1 else r.choice("NS"))
        elif mode == "blocks":
            key = (i // bh, j // bw)
            if key not in cols:
                cols[key] = r.choice("NS")
            s.append(cols[key] if r.random() > 0.05 else r.choice("NS"))
        else:
            s.append("N" if r.random() > 0.03 else "S")
    rows.append("".join(s))
print(n, m)
print("\n".join(rows))
