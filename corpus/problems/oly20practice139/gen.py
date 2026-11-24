"""oly20practice139 inputs: `gen.py seed size` gives 1..max(1, size) pairs 0 <= y <= x, with
values up to 20 (many ties) for even seeds and up to 1e9 for odd ones.
`gen.py --max` gives n = 150000 pairs over the full range."""
import random, sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["1", "150000"]
seed, size = int(sys.argv[1]), int(sys.argv[2])
r = random.Random(seed)
n = size if size >= 150000 else r.randint(1, max(1, size))
hi = 20 if seed % 2 == 0 else 10**9
print(n)
for _ in range(n):
    x = r.randint(0, hi)
    print(x, r.randint(0, x))
