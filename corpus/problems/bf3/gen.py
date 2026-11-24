"""gen.py SEED SIZE | --max: BF 3. One N, 1 <= N <= 2e9. Small cases scale N by size and
sometimes start just past a prime. --max is N = 1294268492, just past 1294268491, which starts
the longest prime gap (288) below 2e9: the most candidates a trial-division search tries."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
print(1_294_268_492 if MAX else r.randint(1, min(2 * 10**9, 10 * S ** 3)))
