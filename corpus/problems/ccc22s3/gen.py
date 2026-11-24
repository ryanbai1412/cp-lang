"""gen.py SEED SIZE | --max: CCC 2022 S3. n notes, m <= n pitches, k good samples wanted, often
near the most a piece can have. --max is n = m = 1e6 with the largest k there is."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10 ** 6 if MAX else r.randint(1, 2 + 3 * S)
m = n if MAX else r.randint(1, n)
most = sum(min(i + 1, m) for i in range(n))
k = most if MAX else r.choice([r.randint(1, most + 2), r.randint(max(1, n - 1), n + 1), most, most + 1])
print(n, m, max(1, k))
