"""gen.py SEED SIZE | --max: An Animal Contest 5 P5 - Counting Rectangles. H horizontal and V
vertical sticks (1 <= H, V <= 5000, coordinates within 1e9, x1 < x2 and y1 < y2). Coordinates come
from a range that grows with size, so small cases have repeated lines, overlaps and sticks that
only touch at an endpoint; some cases use the full 1e9 range. --max is H = V = 5000 sticks that
cross every stick of the other direction, every tenth line repeating its neighbour's coordinate
(zero-area pairs), with endpoints at +-1e9: about 1.5e14 rectangles."""
import random, sys

B = 10**9
if sys.argv[1] == "--max":
    n = 5000
    at = [-B + 400_000 * (i - i % 10 // 9) for i in range(n)]
    print(n, n)
    print("\n".join(f"{a} {-B} {B}" for a in at))
    print("\n".join(f"{a} {-B} {B}" for a in at))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
cap = min(5000, 1 + S)
H, V = r.randint(1, cap), r.randint(1, cap)
R = B if r.random() < 0.15 else min(B, max(2, S // 2 + r.randint(1, 3)))


def stick():
    lo, hi = sorted(r.sample(range(-R, R + 1), 2))
    return f"{r.randint(-R, R)} {lo} {hi}"


print(H, V)
print("\n".join(stick() for _ in range(H + V)))
