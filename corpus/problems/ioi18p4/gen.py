"""ioi18p4 generator: `M n`, then the trigger sequence A[0..n) with values in 1..=M."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 12
m = 100_000 if big else r.randint(1, max(1, size // 2))
n = 200_000 if big else r.randint(1, max(1, size))
print(m, n)
print(*(r.randint(1, m) for _ in range(n)))
