"""gen.py SEED SIZE | --max: CCC 2021 S1 Crazy Fencing. N (N <= 10^4) pieces: N+1 heights and N
widths, each in [1, 100]. --max is N = 10^4 with random heights and widths."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10_000 if MAX else r.randint(1, min(10_000, 1 + S))
print(n)
print(*(r.randint(1, 100) for _ in range(n + 1)))
print(*(r.randint(1, 100) for _ in range(n)))
