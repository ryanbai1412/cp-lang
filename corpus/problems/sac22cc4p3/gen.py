"""sac22cc4p3 (Obligatory Math Problem) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = 1e5).

N (1..=1e5), then N integers in -1e9..=1e9; small sizes use the -100..=100 subtask range,
so ties and even N (many minimizers) are common.
--max: N = 1e5 distinct values spread over the full range.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, a = 10**5, rng.sample(range(-10**9, 10**9 + 1), 10**5)
else:
    size = max(1, int(sys.argv[2]))
    n = rng.randint(1, min(10**5, size))
    lim = 100 if rng.random() < 0.5 else 10**9
    a = [rng.randint(-lim, lim) for _ in range(n)]
print(n)
print(*a)
