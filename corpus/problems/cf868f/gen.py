"""gen.py SEED SIZE | --max: Codeforces 868F (Yet Another Minimization Problem). An array of
n <= 1e5 values in 1..n split into 2 <= k <= min(n, 20) segments. Small cases use few distinct
values. --max is n = 1e5, k = 20, values drawn from 1..300 so every segment has many pairs."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(2, 2 + 2 * S)
k = 20 if MAX else r.randint(2, min(n, 20))
v = 300 if MAX else r.randint(1, n)
a = [r.randint(1, v) for _ in range(n)]
sys.stdout.write(f"{n} {k}\n{' '.join(map(str, a))}\n")
