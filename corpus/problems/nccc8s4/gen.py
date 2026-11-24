"""nccc8s4 generator: m, then m atoms of four side labels in -26..=26 (0 = no join).
--max: m = 1e5 with labels from a small set, so joins and cycles are likely."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
m = 100_000 if big else r.randint(1, 1 + size * 2)
span = r.randint(1, 26)
zero = r.random()
print(m)
for _ in range(m):
    print(*(0 if r.random() < zero else r.randint(-span, span) for _ in range(4)))
