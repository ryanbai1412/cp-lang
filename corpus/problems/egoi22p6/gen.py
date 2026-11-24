"""gen.py SEED SIZE | --max: EGOI '22 P6 (Superpiece). 1 <= q <= 1000 queries, each a
non-empty subsequence of QRBNKP and two distinct squares with |coordinates| <= 1e8. Offsets
are tiny, near a line or diagonal, or at most SIZE apart (capped at 1e8 coordinates).
--max is 1000 queries cycling over all 63 piece sets with far-apart squares."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
C = 10**8
size = C if MAX else max(1, int(sys.argv[2]))
q = 1000 if MAX else r.randint(1, min(1000, size))
sets = ["".join(ch for i, ch in enumerate("QRBNKP") if m >> i & 1) for m in range(1, 64)]
lines = [str(q)]
for t in range(q):
    s = sets[t % 63] if MAX else r.choice(sets)
    span = min(C, size)
    a, b = r.randint(-C, C), r.randint(-C, C)
    while True:
        kind = "far" if MAX else r.choice(["tiny", "line", "far"])
        if kind == "tiny":
            dr, dc = r.randint(-4, 4), r.randint(-4, 4)
        elif kind == "line":
            k = r.randint(-span, span)
            dr, dc = r.choice([(k, 0), (0, k), (k, k), (k, -k)])
            dr += r.randint(-2, 2) * (r.random() < 0.5)
        else:
            dr, dc = r.randint(-span, span), r.randint(-span, span)
        c, d = a + dr, b + dc
        if (dr or dc) and -C <= c <= C and -C <= d <= C:
            break
    lines += [s, f"{a} {b} {c} {d}"]
print("\n".join(lines))
