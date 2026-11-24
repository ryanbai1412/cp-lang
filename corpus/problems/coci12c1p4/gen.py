"""COCI 2012/13 C1 #4 inputs: `gen.py seed size` (m up to `size` colours, n from m up) or
`gen.py --max` (n = 1e9 children, m = 3e5 colours of up to 1e9 marbles).
n stays below the marble total, so the answer is at least 2: the accepted C++ starts its
binary search at 0 and divides by zero when the answer is 1."""
import sys, random
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
m = 300000 if MAX else r.randint(1, max(1, min(300000, int(sys.argv[2]))))
top = 10**9 if MAX else r.choice([5, 1000, 10**9])
marbles = [r.randint(1, top) for _ in range(m)]
marbles[0] = max(marbles[0], 2)
n = 10**9 if MAX else min(r.choice([m, r.randint(m, 3 * m), r.randint(m, 10**9)]), sum(marbles) - 1)
print(n, m)
for x in marbles: print(x)
