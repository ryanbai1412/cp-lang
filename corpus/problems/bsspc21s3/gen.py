"""gen.py SEED SIZE | --max: BSSPC '21 S3. N <= 1e6 values in [-1e9, 1e9] and 1 <= K <= N. Small
cases mix mostly-negative and mixed values. --max is N = 1e6, K = 3, values at the bounds."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1_000_000 if MAX else r.randint(1, min(10**6, 1 + S))
k = 3 if MAX else r.randint(1, n if r.random() < 0.3 else min(n, 4))
hi = 10**9 if MAX or r.random() < 0.5 else 5
top = hi if MAX or r.random() < 0.7 else hi // 3
print(n, k)
print(*(r.randint(-hi, top) for _ in range(n)))
