"""gen.py SEED SIZE | --max: CCC 2023 S3. n, m in 2..=2000, r in 0..=n, c in 0..=m, often at
the extremes 0 and n (m) where the cases split. --max is n = m = 2000 with r, c inside."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 2000 if MAX else r.randint(2, 2 + 2 * S)
m = 2000 if MAX else r.randint(2, 2 + 2 * S)
pick = lambda k: r.choice([0, k, r.randint(0, k), r.randint(0, k), k - 1, 1])
print(n, m, 1000 if MAX else pick(n), 999 if MAX else pick(m))
