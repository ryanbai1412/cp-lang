"""CCO 2023 P2 inputs: `gen.py seed size` (up to `size` columns of small heights, so levels
repeat and valleys nest), or `gen.py --max` (1e6 columns, heights up to 1e9)."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1])); S = 0 if MAX else int(sys.argv[2])
n = 10**6 if MAX else r.randint(1, max(1, S))
top = 10**9 if MAX else r.choice([3, 10, 10**9])
print(n)
print(*(r.randint(1, top) for _ in range(n)))
