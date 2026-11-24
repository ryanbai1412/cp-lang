"""gen.py SEED SIZE | --max: Riolku's Mock CCC S4. N in 1..=5000 buckets with c_i, a_i, b_i in
1..=5000. Small sizes keep values small (the subtask-1 shape) or let b dominate; --max is
N = 5000 with a_i = 1 and b_i = 1 under c_i = 5000, the widest dp rows."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 5000 if MAX else r.randint(1, min(5000, 1 + 2 * S))
hi = 5000 if MAX else r.choice([2, min(5000, 3 + 3 * S), 5000])
print(n)
print(*(5000 if MAX else r.randint(1, hi) for _ in range(n)))
for _ in range(n):
    print(*((1, 1) if MAX else (r.randint(1, hi), r.randint(1, hi))))
