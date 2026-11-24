"""gen.py SEED SIZE | --max: BSSPC '21 S4. An N x M grid (N, M <= 1e9) that is 1 exactly on the union
of K <= 2e5 rectangles. Cases pick a block size 2^j and mostly align rectangles to it, so the
answer is often j; some get one unaligned rectangle. --max is N = M = 2^29 (the most halvings
under 1e9) and K = 2e5 overlapping rectangles aligned to 2^12."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
if MAX:
    n = m = 1 << 29
    k, blk = 200_000, 1 << 12
else:
    j = r.randint(0, 4)
    blk = 1 << j
    n = blk * r.randint(1, max(1, min(1 + S, 10**9 // blk)))
    m = blk * r.randint(1, max(1, min(1 + S, 10**9 // blk)))
    if r.random() < 0.2:
        n += r.randint(0, 1)
    k = r.randint(1, min(200_000, 1 + S))
out = [f"{n} {m} {k}"]
for i in range(k):
    if not MAX and i == 0 and r.random() < 0.3:
        t, b = sorted(r.randint(1, n) for _ in range(2))
        l, rr = sorted(r.randint(1, m) for _ in range(2))
    else:
        bn, bm = n // blk, m // blk
        if bn == 0 or bm == 0:
            t = b = l = rr = 1
        else:
            t, b = sorted(r.randint(0, bn - 1) for _ in range(2))
            l, rr = sorted(r.randint(0, bm - 1) for _ in range(2))
            t, b, l, rr = t * blk + 1, (b + 1) * blk, l * blk + 1, (rr + 1) * blk
    out.append(f"{t} {l} {b} {rr}")
print("\n".join(out))
