import sys, random
# gen.py seed size: n <= 4 size + 2 rows 0 <= L_i <= R_i <= W, consecutive rows overlapping (the
# fossil stays connected), each side a random walk with jumps and plateaus. --max: n = 1e6 rows over
# all of 0..=1e6, sides zigzagging (every row a candidate turn).
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
n = 10**6 if MAX else r.randint(1, 4 * int(sys.argv[2]) + 2)
W = 10**6 if MAX else r.choice([6, 30, 1000])
mid = W // 2; step = max(1, W // 50)
l, h = mid - r.randint(0, step), mid + r.randint(0, step)
out = [str(n)]
for i in range(n):
    out.append(f"{l} {h}")
    nl = l + r.choice([-1, 1]) * r.randint(0, step if r.random() < 0.3 else 2) if r.random() < 0.8 else l
    nh = h + r.choice([-1, 1]) * r.randint(0, step if r.random() < 0.3 else 2) if r.random() < 0.8 else h
    nl, nh = max(0, min(nl, W)), max(0, min(nh, W))
    if nl > nh: nl, nh = nh, nl
    if nh < l: nh = l
    if nl > h: nl = h
    l, h = nl, nh
print("\n".join(out))
