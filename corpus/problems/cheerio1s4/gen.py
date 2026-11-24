"""gen.py SEED SIZE | --max: Cheerio Contest 1 S4 (Mystery Boxes). N <= 5e5 boxes with labels in
1..M, M <= N, every label present. Small cases keep M small or near N. --max is N = 5e5 and
M = 2.5e5 with the boxes shuffled, so each step jumps across the table."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500_000 if MAX else r.randint(1, 1 + 2 * S)
m = n // 2 if MAX else r.choice([1, r.randint(1, n), n, max(1, n // 2)])
a = list(range(1, m + 1)) + [r.randint(1, m) for _ in range(n - m)]
r.shuffle(a)
sys.stdout.write(f"{n} {m}\n{' '.join(map(str, a))}\n")
